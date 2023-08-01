pragma solidity ^0.8.18;

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";

struct PingPongPacket {
    bool ping;
    uint64 counterpartyTimeoutRevisionNumber;
    uint64 counterpartyTimeoutRevisionHeight;
}

library PingPongPacketLib {
    function encode(
        PingPongPacket memory packet
    ) internal pure returns (bytes memory) {
        return
            abi.encode(
                packet.ping,
                packet.counterpartyTimeoutRevisionNumber,
                packet.counterpartyTimeoutRevisionHeight
            );
    }

    function decode(
        bytes memory packet
    ) internal pure returns (PingPongPacket memory) {
        (
            bool ping,
            uint64 counterpartyTimeoutRevisionNumber,
            uint64 counterpartyTimeoutRevisionHeight
        ) = abi.decode(packet, (bool, uint64, uint64));
        return
            PingPongPacket({
                ping: ping,
                counterpartyTimeoutRevisionNumber: counterpartyTimeoutRevisionNumber,
                counterpartyTimeoutRevisionHeight: counterpartyTimeoutRevisionHeight
            });
    }
}

contract PingPong is IBCAppBase {
    using PingPongPacketLib for PingPongPacket;

    IBCHandler private ibcHandler;
    string private portId;
    string private channelId;
    uint64 private revisionNumber;
    uint64 private numberOfBlockBeforePongTimeout;

    event Ring(bool ping);

    constructor(
        IBCHandler _ibcHandler,
        uint64 _revisionNumber,
        uint64 _numberOfBlockBeforePongTimeout
    ) {
        ibcHandler = _ibcHandler;
        revisionNumber = _revisionNumber;
        numberOfBlockBeforePongTimeout = _numberOfBlockBeforePongTimeout;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function initiate(PingPongPacket memory packet) public {
        require(bytes(channelId).length != 0, "channel must be opened");
        ibcHandler.sendPacket(
            portId,
            channelId,
            IbcCoreClientV1Height.Data({
                revision_number: packet.counterpartyTimeoutRevisionNumber,
                revision_height: packet.counterpartyTimeoutRevisionHeight
            }),
            0,
            packet.encode()
        );
    }

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address relayer
    ) external virtual override onlyIBC returns (bytes memory acknowledgement) {
        PingPongPacket memory packet = PingPongPacketLib.decode(packet.data);
        emit Ring(packet.ping);
        packet.ping = !packet.ping;
        packet.counterpartyTimeoutRevisionNumber = revisionNumber;
        packet.counterpartyTimeoutRevisionHeight =
            uint64(block.number) +
            numberOfBlockBeforePongTimeout;
        initiate(packet);
        return hex"01";
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override onlyIBC {}

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata
    ) external virtual override onlyIBC {
        require(bytes(channelId).length == 0, "only one channel can be opened");
    }

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        string calldata
    ) external virtual override onlyIBC {
        require(bytes(channelId).length == 0, "only one channel can be opened");
    }

    function onChanOpenAck(
        string calldata _portId,
        string calldata _channelId,
        string calldata
    ) external virtual override onlyIBC {
        portId = _portId;
        channelId = _channelId;
    }

    function onChanOpenConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {
        portId = _portId;
        channelId = _channelId;
    }

    function onChanCloseInit(
        string calldata,
        string calldata
    ) external virtual override onlyIBC {
        revert("This game is infinite");
    }

    function onChanCloseConfirm(
        string calldata,
        string calldata
    ) external virtual override onlyIBC {
        revert("This game is infinite");
    }
}