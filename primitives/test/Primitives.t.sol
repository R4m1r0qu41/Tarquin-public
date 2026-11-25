// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "../src/core/Types.sol";
import {LegalHold} from "../src/primitives/LegalHold.sol";
import {EscrowRelease} from "../src/primitives/EscrowRelease.sol";

/// @notice Foundry tests for the primitive stubs. No forge-std dependency: a test passes if it
///         does not revert. These assert the same semantics the engine rules encode
///         (`crates/tarquin-engine`) — the on-chain side of dual-target parity.
contract PrimitivesTest {
    LegalHold internal hold;
    EscrowRelease internal escrow;

    function setUp() public {
        hold = new LegalHold(bytes32("legal_hold_v0"));
        escrow = new EscrowRelease(bytes32("escrow_release_v0"));
    }

    function _disp(Types.Action p, Types.Confidence c) internal pure returns (Types.Disposition memory d) {
        d.action = p;
        d.confidence = c;
        d.finality = Types.Finality.IRREVERSIBLE;
        d.subtype = Types.LegSubtype.JUD;
    }

    // ── LegalHold: a restriction never permits PR (mirrors engine default/HL path) ──

    function test_LegalHold_rejects_PR() public view {
        require(!hold.permits(_disp(Types.Action.PR, Types.Confidence.C), ""), "hold must reject PR");
    }

    function test_LegalHold_allows_hold() public view {
        require(hold.permits(_disp(Types.Action.HL, Types.Confidence.C), ""), "hold allows HL");
        require(hold.permits(_disp(Types.Action.BL, Types.Confidence.C), ""), "hold allows BL");
    }

    // ── EscrowRelease: permits ONLY on a PR disposition (mirrors ENG-PR-ELEVATION) ──

    function test_EscrowRelease_allows_PR() public view {
        require(escrow.permits(_disp(Types.Action.PR, Types.Confidence.C), ""), "escrow allows PR");
    }

    function test_EscrowRelease_rejects_non_PR() public view {
        require(!escrow.permits(_disp(Types.Action.HL, Types.Confidence.C), ""), "escrow rejects HL");
        require(!escrow.permits(_disp(Types.Action.BL, Types.Confidence.C), ""), "escrow rejects BL");
    }

    function test_EscrowRelease_rejects_confidence_A() public view {
        // Confidence A is never valid for a LEG disposition (OA-006), even with PR.
        require(!escrow.permits(_disp(Types.Action.PR, Types.Confidence.A), ""), "escrow rejects confidence A");
    }
}
