// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title Tarquin on-chain mirror of the Arrow six-field classification model.
/// @notice These enums mirror `Class:Origin:State:Action:Finality:Confidence`.
///         They are intentionally minimal: the on-chain leg ENFORCES a disposition,
///         it does not DERIVE one. Derivation lives in Lex.exe + Arrow Step 5b.
/// @dev See specs/leg-origin-amendment.md and lex-exe/leg-origin-adapter.md.
library Types {
    /// @notice Arrow Action field. Tarquin runtime is fail-closed: HOLD/BLOCK are safe.
    enum Action {
        UNSET, // 0 — never a valid stored disposition; reverts if encountered
        BL, // Block
        HL, // Hold for human review (fail-closed default)
        RW, // Rework / return
        TR, // Transfer (compelled custody move; see RFC-007 §13 Q3)
        PR // Proceed / execute (reachable only via full conjunctive gate)
    }

    /// @notice Binary finality axis. 0 = settled-but-challengeable (FN+0); 1 = irreversible.
    enum Finality {
        CHALLENGEABLE, // 0
        IRREVERSIBLE // 1
    }

    /// @notice Confidence ceiling. LEG-origin events are capped at B (no A on-chain).
    enum Confidence {
        UNSET, // 0
        E,
        D,
        C,
        B,
        A // reserved for native_canonical; MUST NOT appear on a LEG disposition
    }

    /// @notice LEG subtype carried for audit; not used for on-chain branching in v0.1.
    enum LegSubtype {
        UNSET,
        JUD,
        ADM,
        SUPRA,
        LEG_TEXT,
        PREC
    }

    /// @notice The minimal disposition record the gate stores and enforces.
    struct Disposition {
        Action action;
        Finality finality;
        Confidence confidence;
        LegSubtype subtype;
        bytes32 proofRefHash; // commitment to the LBRY record (proof_ref)
        bytes32 assertedBy; // hash of the attesting Sibyl DID
        bytes32 arrowEventId; // the Arrow event this disposition resolves
    }
}
