import { readFileSync } from "fs";
import { execSync } from "child_process";

function aliceLnCli(command: string): any {
    const result = execSync(`docker exec alice lightning-cli --network=regtest ${command}`, {
        encoding: 'utf-8'
    });
    return JSON.parse(result);
}

function bobLnCli(command: string): any {
    const result = execSync(`docker exec bob lightning-cli --network=regtest ${command}`, {
        encoding: 'utf-8'
    });
    return JSON.parse(result);
}

describe('Evaluate submission', () => {
    let aliceNodeId: string;
    let bobNodeId: string;
    let alicePeerCount: number;
    let bobPeerCount: number;
    let channelId: string;
    let fundingTxid: string;
    let aliceChannelState: string;
    let bobChannelState: string;
    let totalMsat: number;
    let aliceBalanceMsat: number;
    let bobBalanceMsat: number;

    it('should read and parse out.txt', () => {
        const lines = readFileSync('out.txt', 'utf-8').trim().split('\n');
        expect(lines).toHaveLength(11);

        aliceNodeId = lines[0].trim();
        bobNodeId = lines[1].trim();
        alicePeerCount = parseInt(lines[2].trim(), 10);
        bobPeerCount = parseInt(lines[3].trim(), 10);
        channelId = lines[4].trim();
        fundingTxid = lines[5].trim();
        aliceChannelState = lines[6].trim();
        bobChannelState = lines[7].trim();
        totalMsat = parseInt(lines[8].trim(), 10);
        aliceBalanceMsat = parseInt(lines[9].trim(), 10);
        bobBalanceMsat = parseInt(lines[10].trim(), 10);

        expect(aliceNodeId).toBeTruthy();
        expect(bobNodeId).toBeTruthy();
        expect(alicePeerCount).toBeDefined();
        expect(bobPeerCount).toBeDefined();
        expect(channelId).toBeTruthy();
        expect(fundingTxid).toBeTruthy();
        expect(aliceChannelState).toBeTruthy();
        expect(bobChannelState).toBeTruthy();
        expect(totalMsat).toBeDefined();
        expect(aliceBalanceMsat).toBeDefined();
        expect(bobBalanceMsat).toBeDefined();
    });

    it('should have valid node ID format (66 hex characters)', () => {
        expect(aliceNodeId).toMatch(/^[a-f0-9]{66}$/);
        expect(aliceNodeId.length).toBe(66);
        expect(bobNodeId).toMatch(/^[a-f0-9]{66}$/);
        expect(bobNodeId.length).toBe(66);
    });

    it('should have node IDs matching actual node IDs', () => {
        const aliceInfo = aliceLnCli('getinfo');
        const bobInfo = bobLnCli('getinfo');

        expect(aliceNodeId).toBe(aliceInfo.id);
        expect(bobNodeId).toBe(bobInfo.id);
    });

    it('should have peer connections', () => {
        expect(alicePeerCount).toBeGreaterThan(0);
        expect(bobPeerCount).toBeGreaterThan(0);
    });

    it('should have valid channel ID and funding txid format', () => {
        expect(channelId).toMatch(/^[a-f0-9]+$/);
        expect(fundingTxid).toMatch(/^[a-f0-9]{64}$/);
        expect(fundingTxid.length).toBe(64);
    });

    it('should have channel state as CHANNELD_NORMAL', () => {
        expect(aliceChannelState).toBe('CHANNELD_NORMAL');
        expect(bobChannelState).toBe('CHANNELD_NORMAL');
    });

    it('should have 500,000 satoshis total capacity', () => {
        expect(totalMsat).toBe(500000000); // 500,000 satoshis = 500,000,000 millisatoshis
    });

    it('should have Alice with most balance and Bob with zero', () => {
        expect(aliceBalanceMsat).toBeGreaterThan(490000000); // Should have most of 500M msat (minus fees)
        expect(bobBalanceMsat).toBe(0); // Bob should have 0 initially
    });

    it('should have Alice connected to Bob as peer', () => {
        const alicePeers = aliceLnCli('listpeers');
        const bobInfo = bobLnCli('getinfo');

        expect(alicePeers.peers).toBeDefined();
        expect(alicePeers.peers.length).toBeGreaterThan(0);

        const bobPeer = alicePeers.peers.find((peer: any) => peer.id === bobInfo.id);
        expect(bobPeer).toBeDefined();
        expect(bobPeer.connected).toBe(true);
    });

    it('should have Bob connected to Alice as peer', () => {
        const bobPeers = bobLnCli('listpeers');
        const aliceInfo = aliceLnCli('getinfo');

        expect(bobPeers.peers).toBeDefined();
        expect(bobPeers.peers.length).toBeGreaterThan(0);

        const alicePeer = bobPeers.peers.find((peer: any) => peer.id === aliceInfo.id);
        expect(alicePeer).toBeDefined();
        expect(alicePeer.connected).toBe(true);
    });

    it('should have valid channel on Alice side', () => {
        const aliceChannels = aliceLnCli('listpeerchannels');
        const bobInfo = bobLnCli('getinfo');

        expect(aliceChannels.channels).toBeDefined();
        expect(aliceChannels.channels.length).toBeGreaterThan(0);

        const channel = aliceChannels.channels.find((ch: any) => ch.peer_id === bobInfo.id);
        expect(channel).toBeDefined();
        expect(channel.total_msat).toBe(500000000);
        expect(channel.state).toBe('CHANNELD_NORMAL');
        expect(channel.funding_txid).toBe(fundingTxid);
    });

    it('should have valid channel on Bob side', () => {
        const bobChannels = bobLnCli('listpeerchannels');
        const aliceInfo = aliceLnCli('getinfo');

        expect(bobChannels.channels).toBeDefined();
        expect(bobChannels.channels.length).toBeGreaterThan(0);

        const channel = bobChannels.channels.find((ch: any) => ch.peer_id === aliceInfo.id);
        expect(channel).toBeDefined();
        expect(channel.total_msat).toBe(500000000);
        expect(channel.state).toBe('CHANNELD_NORMAL');
        expect(channel.funding_txid).toBe(fundingTxid);
    });

    it('should have both sides with same funding transaction ID', () => {
        const aliceChannels = aliceLnCli('listpeerchannels');
        const bobChannels = bobLnCli('listpeerchannels');
        const aliceInfo = aliceLnCli('getinfo');
        const bobInfo = bobLnCli('getinfo');

        const aliceChannel = aliceChannels.channels.find((ch: any) => ch.peer_id === bobInfo.id);
        const bobChannel = bobChannels.channels.find((ch: any) => ch.peer_id === aliceInfo.id);

        expect(aliceChannel).toBeDefined();
        expect(bobChannel).toBeDefined();
        expect(aliceChannel.funding_txid).toBe(bobChannel.funding_txid);
    });

    it('should have Alice with full channel balance', () => {
        const aliceChannels = aliceLnCli('listpeerchannels');
        const bobInfo = bobLnCli('getinfo');

        const channel = aliceChannels.channels.find((ch: any) => ch.peer_id === bobInfo.id);

        expect(channel).toBeDefined();
        expect(channel.to_us_msat).toBe(500000000);
        expect(channel.total_msat).toBe(500000000);
        expect(channel.spendable_msat).toBeGreaterThan(490000000);
        expect(channel.spendable_msat).toBeLessThan(500000000);
        expect(channel.receivable_msat).toBe(0);
    });

    it('should have Bob with zero balance in channel', () => {
        const bobChannels = bobLnCli('listpeerchannels');
        const aliceInfo = aliceLnCli('getinfo');

        const channel = bobChannels.channels.find((ch: any) => ch.peer_id === aliceInfo.id);

        expect(channel).toBeDefined();
        expect(channel.to_us_msat).toBe(0);
        expect(channel.total_msat).toBe(500000000);
        expect(channel.spendable_msat).toBe(0);
        expect(channel.receivable_msat).toBeGreaterThan(490000000);
    });
});
