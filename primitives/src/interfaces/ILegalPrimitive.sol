// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "../core/Types.sol";

/// @title ILegalPrimitive — base interface every Tarquin SLC primitive implements.
/// @notice A primitive is an audited, single-purpose on-chain legal building block.
///         It is the on-chain counterpart of an off-chain engine clause and MUST stay
///         in deterministic parity with it (ADR-0001, Decision 2).
interface ILegalPrimitive {
    /// @notice Stable identifier of the off-chain engine clause this primitive mirrors.
    /// @dev Used by the parity-test harness (PR-003) to bind the two targets.
    function parityRef() external view returns (bytes32);

    /// @notice The Lex.Codex permits() gate, evaluated on-chain.
    /// @dev Pure/deterministic over (disposition, ctx). MUST NOT call any oracle that
    ///      performs inference. Returns false to veto; a false result is non-overridable.
    /// @param disposition the Arrow-derived, human-ratified disposition
    /// @param ctx ABI-encoded primitive-specific context (no inference inputs)
    function permits(Types.Disposition calldata disposition, bytes calldata ctx) external view returns (bool);
}
