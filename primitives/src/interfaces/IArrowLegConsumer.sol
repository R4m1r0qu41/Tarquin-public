// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Types} from "../core/Types.sol";

/// @title IArrowLegConsumer — on-chain surface of the fail-closed `arrow-consume-leg` consumer.
/// @notice Mirrors lex-exe/leg-consumer.md. Every disposition is QUARANTINED on receipt;
///         nothing executes until a human ratifier confirms. There is no autonomous path.
interface IArrowLegConsumer {
    /// @notice A disposition was received from Arrow and placed in quarantine.
    event Quarantined(bytes32 indexed arrowEventId, Types.Action action);
    /// @notice A human ratifier approved a quarantined disposition for execution.
    event Ratified(bytes32 indexed arrowEventId, address indexed ratifier);
    /// @notice A quarantined disposition was rejected (fail-closed terminal state).
    event Rejected(bytes32 indexed arrowEventId, address indexed reviewer, string reason);

    /// @notice Ingest an Arrow disposition. MUST place it in quarantine; MUST NOT execute.
    function receiveDisposition(Types.Disposition calldata disposition) external;

    /// @notice Human-ratification gate. Only an authorized ratifier may call.
    ///         MUST verify the disposition is unchanged since quarantine before executing.
    function ratify(bytes32 arrowEventId) external;

    /// @notice Terminal rejection of a quarantined disposition.
    function reject(bytes32 arrowEventId, string calldata reason) external;

    /// @notice True only after ratification AND successful gate checks. Default: false.
    function isExecutable(bytes32 arrowEventId) external view returns (bool);
}
