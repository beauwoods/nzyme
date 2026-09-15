package app.nzyme.core.rest.resources.ethernet;

import app.nzyme.core.NzymeNode;
import app.nzyme.core.database.OrderDirection;
import app.nzyme.core.ethernet.L4Type;
import app.nzyme.core.ethernet.webrtc.WebRTC;
import app.nzyme.core.ethernet.webrtc.db.WebRTCRTPStreamData;
import app.nzyme.core.ethernet.webrtc.db.WebRTCSessionEntry;
import app.nzyme.core.rest.RestHelpers;
import app.nzyme.core.rest.TapDataHandlingResource;
import app.nzyme.core.rest.responses.ethernet.L4AddressResponse;
import app.nzyme.core.rest.responses.ethernet.webrtc.WebRTCRTPStreamDetailsResponse;
import app.nzyme.core.rest.responses.ethernet.webrtc.WebRTCSessionDetailsResponse;
import app.nzyme.core.rest.responses.ethernet.webrtc.WebRTCSessionsListResponse;
import app.nzyme.core.util.TimeRange;
import app.nzyme.core.util.filters.Filters;
import app.nzyme.plugin.rest.security.PermissionLevel;
import app.nzyme.plugin.rest.security.RESTSecured;
import com.google.common.collect.Lists;
import jakarta.annotation.Nullable;
import jakarta.inject.Inject;
import jakarta.validation.Valid;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Context;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.SecurityContext;
import tools.jackson.core.type.TypeReference;
import tools.jackson.databind.ObjectMapper;

import java.util.List;
import java.util.UUID;

import static app.nzyme.core.util.filters.FilterParser.parseFiltersQueryParameter;

@Path("/api/ethernet/webrtc")
@Produces(MediaType.APPLICATION_JSON)
@RESTSecured(PermissionLevel.ANY)
public class WebRTCResource extends TapDataHandlingResource {

    @Inject
    private NzymeNode nzyme;

    @Inject
    private ObjectMapper om;

    @GET
    @Path("/sessions")
    public Response allSessions(@Context SecurityContext sc,
                                @QueryParam("organization_id") UUID organizationId,
                                @QueryParam("tenant_id") UUID tenantId,
                                @QueryParam("time_range") @Valid String timeRangeParameter,
                                @QueryParam("filters") String filtersParameter,
                                @QueryParam("order_column") @Nullable String orderColumnParam,
                                @QueryParam("order_direction") @Nullable String orderDirectionParam,
                                @QueryParam("limit") int limit,
                                @QueryParam("offset") int offset,
                                @QueryParam("taps") String tapIds) {

        List<UUID> taps = parseAndValidateTapIds(getAuthenticatedUser(sc), nzyme, tapIds);
        TimeRange timeRange = parseTimeRangeQueryParameter(timeRangeParameter);
        Filters filters = parseFiltersQueryParameter(filtersParameter);

        if (!passedTenantDataAccessible(sc, organizationId, tenantId)) {
            return Response.status(Response.Status.NOT_FOUND).build();
        }

        WebRTC.SessionOrderColumn orderColumn = WebRTC.SessionOrderColumn.INITIATED_AT;
        OrderDirection orderDirection = OrderDirection.DESC;
        if (orderColumnParam != null && orderDirectionParam != null) {
            try {
                orderColumn = WebRTC.SessionOrderColumn.valueOf(orderColumnParam.toUpperCase());
                orderDirection = OrderDirection.valueOf(orderDirectionParam.toUpperCase());
            } catch (IllegalArgumentException e) {
                return Response.status(Response.Status.BAD_REQUEST).build();
            }
        }

        long total = nzyme.getEthernet().webRtc().countAllSessions(timeRange, filters, taps);

        List<WebRTCSessionDetailsResponse> sessions = Lists.newArrayList();
        for (WebRTCSessionEntry session : nzyme.getEthernet().webRtc()
                .findAllSessions(timeRange, filters, orderColumn, orderDirection, limit, offset, taps)) {

            List<WebRTCRTPStreamDetailsResponse> rtpStreams = Lists.newArrayList();
            for (WebRTCRTPStreamData s : om.readValue(session.rtpStreams(), new TypeReference<List<WebRTCRTPStreamData>>() {})) {
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

            sessions.add(WebRTCSessionDetailsResponse.create(
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
                    session.durationMs(),
                    session.firstSeen(),
                    session.lastActivity()
            ));
        }

        return Response.ok(WebRTCSessionsListResponse.create(total, sessions)).build();
    }

}
