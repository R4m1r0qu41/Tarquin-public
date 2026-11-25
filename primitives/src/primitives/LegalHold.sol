// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "../core/Types.sol";
import {ILegalPrimitive} from "../interfaces/ILegalPrimitive.sol";

/// @title LegalHold — example Class 3 (CONDITION/Restrict) primitive (STUB).
/// @notice Models a legal hold/immobilization installed by a recognized authority.
///         A hold is restrictive: it can BLOCK or HOLD, never PROCEED. This is the
///         simplest worked primitive and the template for the parity-test harness.
/// @dev Mirrors an off-chain engine clause identified by `parityRef()` (PR-003).
contract LegalHold is ILegalPrimitive {
    bytes32 private immutable _parityRef;

    constructor(bytes32 parityRef_) {
        _parityRef = parityRef_;
    }

    /// @inheritdoc ILegalPrimitive
    function parityRef() external view returns (bytes32) {
        return _parityRef;
    }

    /// @inheritdoc ILegalPrimitive
    /// @notice A legal hold NEVER permits the proposed action while active. It is a
    ///         restriction primitive: permits() returns false for any PR attempt.
    function permits(
        Types.Disposition calldata disposition,
        bytes calldata /*ctx*/
    )
        external
        pure
        returns (bool)
    {
        // Restrictive by construction: a hold cannot authorize PR.
        if (disposition.action == Types.Action.PR) {
            return false;
        }
        // HL / RW / BL are the only coherent dispositions under an active hold.
        return true;
    }
}
