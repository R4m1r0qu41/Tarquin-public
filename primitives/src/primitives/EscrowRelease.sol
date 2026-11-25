// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "../core/Types.sol";
import {ILegalPrimitive} from "../interfaces/ILegalPrimitive.sol";

/// @title EscrowRelease — example PR-gated primitive (STUB).
/// @notice The on-chain counterpart of the engine rule `ENG-PR-ELEVATION`
///         (`crates/tarquin-engine`). It permits the proposed action ONLY when the disposition
///         is `PR` — i.e. the full conjunctive gate (OA-011 / LC-006) already passed off-chain
///         and a human ratified. Anything else is refused (fail-closed).
/// @dev parityRef binds this to the engine rule for the parity harness (PR-003).
contract EscrowRelease is ILegalPrimitive {
    bytes32 private immutable _parityRef;

    constructor(bytes32 parityRef_) {
        _parityRef = parityRef_;
    }

    /// @inheritdoc ILegalPrimitive
    function parityRef() external view returns (bytes32) {
        return _parityRef;
    }

    /// @inheritdoc ILegalPrimitive
    /// @notice Permits release only on an explicit PR disposition. Confidence A is never valid
    ///         for a LEG disposition (OA-006), so it is refused even with PR.
    function permits(
        Types.Disposition calldata disposition,
        bytes calldata /*ctx*/
    )
        external
        pure
        returns (bool)
    {
        if (disposition.action != Types.Action.PR) {
            return false;
        }
        if (disposition.confidence == Types.Confidence.A) {
            return false;
        }
        return true;
    }
}
