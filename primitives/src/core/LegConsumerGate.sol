// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "./Types.sol";
import {IArrowLegConsumer} from "../interfaces/IArrowLegConsumer.sol";

/// @title LegConsumerGate — abstract fail-closed + human-ratification gate (STUB).
/// @notice Reference skeleton for the on-chain side of `arrow-consume-leg`.
///         Concrete deployments wire authorization and execution. This stub encodes the
///         invariants only; it intentionally leaves access control abstract.
/// @dev Companion spec: lex-exe/leg-consumer.md.
abstract contract LegConsumerGate is IArrowLegConsumer {
    enum Status {
        NONE,
        QUARANTINED,
        RATIFIED,
        REJECTED,
        EXECUTED
    }

    mapping(bytes32 => Types.Disposition) internal _disposition;
    mapping(bytes32 => Status) internal _status;

    error NotQuarantined(bytes32 id);
    error NotRatifier(address who);
    error ActionNotProceed(Types.Action action);
    error DispositionMutated(bytes32 id);

    /// @dev Deployments implement the ratifier authorization policy.
    function _isRatifier(address who) internal view virtual returns (bool);

    /// @dev Deployments implement the actual on-chain effect after the gate passes.
    function _execute(Types.Disposition memory d) internal virtual;

    modifier onlyRatifier() {
        if (!_isRatifier(msg.sender)) revert NotRatifier(msg.sender);
        _;
    }

    /// @inheritdoc IArrowLegConsumer
    function receiveDisposition(Types.Disposition calldata d) external virtual {
        // Fail-closed: store and quarantine. NEVER execute on receipt.
        bytes32 id = d.arrowEventId;
        _disposition[id] = d;
        _status[id] = Status.QUARANTINED;
        emit Quarantined(id, d.action);
    }

    /// @inheritdoc IArrowLegConsumer
    function ratify(bytes32 id) external virtual onlyRatifier {
        if (_status[id] != Status.QUARANTINED) revert NotQuarantined(id);
        Types.Disposition memory d = _disposition[id];

        // The on-chain leg ENFORCES; it does not re-derive. Only an explicit PR executes.
        // Everything else stays quarantined / terminal — fail-closed.
        if (d.action != Types.Action.PR) revert ActionNotProceed(d.action);

        // Confidence A is never valid for a LEG disposition (OA-006).
        require(d.confidence != Types.Confidence.A, "A not permitted for LEG");

        _status[id] = Status.RATIFIED;
        emit Ratified(id, msg.sender);

        _status[id] = Status.EXECUTED;
        _execute(d);
    }

    /// @inheritdoc IArrowLegConsumer
    function reject(bytes32 id, string calldata reason) external virtual onlyRatifier {
        if (_status[id] != Status.QUARANTINED) revert NotQuarantined(id);
        _status[id] = Status.REJECTED; // terminal
        emit Rejected(id, msg.sender, reason);
    }

    /// @inheritdoc IArrowLegConsumer
    function isExecutable(bytes32 id) external view virtual returns (bool) {
        return _status[id] == Status.EXECUTED;
    }
}
