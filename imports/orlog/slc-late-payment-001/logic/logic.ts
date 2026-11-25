/*
 * slc-late-payment-001 - deterministic contract logic.
 * Pure functions only: no I/O, no clock, no randomness, no model calls (ADR-0001).
 * Self-contained by design; implements prim-grace-period, prim-late-fee-interest,
 * prim-remedy-cap conventions.
 */

export type AccrualBasis = "FROM_DUE_DATE" | "FROM_GRACE_EXPIRY";
export type ChargeKind = "ANNUAL_RATE" | "PER_DAY_AMOUNT";
export type CapKind = "ABSOLUTE_AMOUNT" | "PCT_OF_REFERENCE";

export interface LatePaymentTerms {
  principal: number;
  currency: string;
  dueDate: string; // ISO YYYY-MM-DD
  graceDays: number;
  accrualBasis: AccrualBasis;
  chargeKind: ChargeKind;
  rateOrAmount: number;
  capKind: CapKind;
  capValue: number;
}

export interface PaymentEvent {
  paidDate: string; // ISO YYYY-MM-DD; date payment in full was received
  amountPaid: number;
}

export interface LatePaymentResult {
  daysLate: number;
  chargeableDays: number;
  lateCharge: number;
  capped: boolean;
  totalDue: number;
  obligation: "NONE" | "LATE_CHARGE_DUE";
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

/** UTC day number for an ISO date; throws RangeError on malformed/impossible dates. */
export function dayNumber(isoDate: string): number {
  if (!ISO_DATE.test(isoDate)) {
    throw new RangeError(`invalid ISO date: ${isoDate}`);
  }
  const [y, m, d] = isoDate.split("-").map(Number);
  const ms = Date.UTC(y, m - 1, d);
  const check = new Date(ms);
  if (
    check.getUTCFullYear() !== y ||
    check.getUTCMonth() !== m - 1 ||
    check.getUTCDate() !== d
  ) {
    throw new RangeError(`impossible calendar date: ${isoDate}`);
  }
  return ms / 86_400_000;
}

/** Round half-up to 2 decimals, once, at the end of a computed amount. */
export function round2(x: number): number {
  return Math.round((x + Number.EPSILON) * 100) / 100;
}

function assertNonNegative(name: string, v: number): void {
  if (!Number.isFinite(v) || v < 0) {
    throw new RangeError(`${name} must be a finite non-negative number, got ${v}`);
  }
}

/** prim-grace-period: days that count toward the charge. */
export function chargeableDays(
  dueDate: string,
  performedDate: string,
  graceDays: number,
  basis: AccrualBasis
): number {
  if (!Number.isInteger(graceDays) || graceDays < 0) {
    throw new RangeError(`graceDays must be a non-negative integer, got ${graceDays}`);
  }
  const due = dayNumber(dueDate);
  const performed = dayNumber(performedDate);
  if (performed <= due + graceDays) return 0;
  if (basis === "FROM_DUE_DATE") return performed - due;
  return performed - (due + graceDays);
}

/** prim-late-fee-interest: charge before cap. ANNUAL_RATE is simple ACT/365F. */
export function lateCharge(
  baseAmount: number,
  days: number,
  kind: ChargeKind,
  rateOrAmount: number
): number {
  assertNonNegative("baseAmount", baseAmount);
  assertNonNegative("rateOrAmount", rateOrAmount);
  if (!Number.isInteger(days) || days < 0) {
    throw new RangeError(`days must be a non-negative integer, got ${days}`);
  }
  if (days === 0) return 0;
  if (kind === "ANNUAL_RATE") {
    return round2((baseAmount * (rateOrAmount / 100) * days) / 365);
  }
  return round2(rateOrAmount * days);
}

/** prim-remedy-cap: cap never increases a charge. */
export function applyCap(
  computedCharge: number,
  kind: CapKind,
  capValue: number,
  referenceAmount: number
): { charge: number; capped: boolean } {
  assertNonNegative("computedCharge", computedCharge);
  assertNonNegative("capValue", capValue);
  assertNonNegative("referenceAmount", referenceAmount);
  const limit =
    kind === "ABSOLUTE_AMOUNT" ? capValue : round2((referenceAmount * capValue) / 100);
  if (computedCharge > limit) return { charge: limit, capped: true };
  return { charge: computedCharge, capped: false };
}

/** Contract logic: process a payment event against the agreed terms. */
export function evaluateLatePayment(
  terms: LatePaymentTerms,
  event: PaymentEvent
): LatePaymentResult {
  assertNonNegative("principal", terms.principal);
  assertNonNegative("amountPaid", event.amountPaid);
  if (round2(event.amountPaid) !== round2(terms.principal)) {
    throw new RangeError(
      `amountPaid must equal principal for this payment-in-full artifact; partial payments are not modeled`
    );
  }
  const due = dayNumber(terms.dueDate);
  const paid = dayNumber(event.paidDate);
  const daysLate = Math.max(0, paid - due);
  const days = chargeableDays(
    terms.dueDate,
    event.paidDate,
    terms.graceDays,
    terms.accrualBasis
  );
  const raw = lateCharge(terms.principal, days, terms.chargeKind, terms.rateOrAmount);
  const { charge, capped } = applyCap(raw, terms.capKind, terms.capValue, terms.principal);
  return {
    daysLate,
    chargeableDays: days,
    lateCharge: charge,
    capped,
    totalDue: round2(terms.principal + charge),
    obligation: charge > 0 ? "LATE_CHARGE_DUE" : "NONE",
  };
}
