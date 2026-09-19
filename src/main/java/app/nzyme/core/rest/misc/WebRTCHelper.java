package app.nzyme.core.rest.misc;

import app.nzyme.core.NzymeNode;
import app.nzyme.core.ethernet.L4Type;
import app.nzyme.core.ethernet.webrtc.db.WebRTCRTPStreamData;
import app.nzyme.core.ethernet.webrtc.db.WebRTCSessionEntry;
import app.nzyme.core.rest.RestHelpers;
import app.nzyme.core.rest.responses.ethernet.L4AddressResponse;
import app.nzyme.core.rest.responses.ethernet.nat.NATSTUNNegotiationDetailsResponse;
import app.nzyme.core.rest.responses.ethernet.webrtc.WebRTCRTPStreamDetailsResponse;
import app.nzyme.core.rest.responses.ethernet.webrtc.WebRTCSessionDetailsResponse;
import com.google.common.collect.Lists;
import jakarta.annotation.Nullable;
import tools.jackson.core.type.TypeReference;
import tools.jackson.databind.ObjectMapper;

import java.util.List;
import java.util.UUID;

public class WebRTCHelper {

    private static final ObjectMapper OM = new ObjectMapper();

    public static WebRTCSessionDetailsResponse buildWebRTCSessionDetailsResponse(WebRTCSessionEntry session,
                                                                                 @Nullable List<WebRTCSessionDetailsResponse> subSessions,
                                                                                 @Nullable NATSTUNNegotiationDetailsResponse stunNegotiation,
                                                                                 NzymeNode nzyme,
                                                                                 UUID organizationId,
                                                                                 UUID tenantId) {
        List<WebRTCRTPStreamDetailsResponse> rtpStreams = Lists.newArrayList();
        for (WebRTCRTPStreamData s : OM.readValue(session.rtpStreams(), new TypeReference<List<WebRTCRTPStreamData>>() {})) {
            rtpStreams.add(WebRTCRTPStreamDetailsResponse.create(
                    s.ssrc(),
                    s.direction(),
                    s.payloadTypes(),
                    s.packetCount(),
                    s.byteCount(),
                    s.mediaKind(),
                    s.estimatedLost(),
                    s.sequenceMin(),
                    s.sequenceMax(),
                    s.timestampSpan(),
                    s.markerCount()
            ));
        }

        L4AddressResponse source;
        if (session.source() != null) {
            source = RestHelpers.L4AddressDataToResponse(
                    nzyme,
                    organizationId,
                    tenantId,
                    L4Type.UDP,
                    session.source()
            );
        } else {
            source = null;
        }

        L4AddressResponse destination;
        if (session.destination() != null) {
            destination = RestHelpers.L4AddressDataToResponse(
                    nzyme,
                    organizationId,
                    tenantId,
                    L4Type.UDP,
                    session.destination()
            );
        } else {
            destination = null;
        }

        return WebRTCSessionDetailsResponse.create(
                session.negotiationKey(),
                session.negotiationKeySha256(),
                session.transport(),
                session.isActive(),
                session.hasRtp(),
                session.hasDtls(),
                session.hasAudio(),
                session.hasVideo(),
                session.streamCount(),
                session.dtlsAppDataRecords(),
                session.bytesExchanged(),
                source,
                destination,
                rtpStreams,
                subSessions,
                stunNegotiation,
                session.durationMs(),
                session.firstSeen(),
                session.lastActivity()
        );
    }

}
