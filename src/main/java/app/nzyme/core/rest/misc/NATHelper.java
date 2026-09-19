package app.nzyme.core.rest.misc;

import app.nzyme.core.NzymeNode;
import app.nzyme.core.ethernet.L4Type;
import app.nzyme.core.ethernet.nat.db.STUNNegotiationEntry;
import app.nzyme.core.rest.RestHelpers;
import app.nzyme.core.rest.responses.ethernet.L4AddressResponse;
import app.nzyme.core.rest.responses.ethernet.nat.NATSTUNNegotiationDetailsResponse;
import jakarta.annotation.Nullable;

import java.util.List;
import java.util.Map;
import java.util.UUID;

public class NATHelper {

    public static NATSTUNNegotiationDetailsResponse buildNegotiationDetailsResponse(STUNNegotiationEntry negotiation,
                                                                                    List<NATSTUNNegotiationDetailsResponse> flows,
                                                                                    @Nullable Map<String, Object> relatedConnections,
                                                                                    NzymeNode nzyme,
                                                                                    UUID organizationId,
                                                                                    UUID tenantId) {
        L4AddressResponse source = null;
        if (negotiation.source() != null) {
            source = RestHelpers.L4AddressDataToResponse(
                    nzyme,
                    organizationId,
                    tenantId,
                    L4Type.valueOf(negotiation.transport().toUpperCase()),
                    negotiation.source()
            );
        }

        L4AddressResponse destination = null;
        if (negotiation.destination() != null) {
            destination = RestHelpers.L4AddressDataToResponse(
                    nzyme,
                    organizationId,
                    tenantId,
                    L4Type.valueOf(negotiation.transport().toUpperCase()),
                    negotiation.destination()
            );
        }

        List<L4AddressResponse> mappedAddresses = negotiation.mappedAddresses()
                .stream()
                .map(ma -> RestHelpers.L4AddressDataToResponse(
                        nzyme,
                        organizationId,
                        tenantId,
                        L4Type.valueOf(negotiation.transport().toUpperCase()), ma))
                .toList();

        List<L4AddressResponse> peerAddresses = negotiation.peerAddresses()
                .stream()
                .map(ma -> RestHelpers.L4AddressDataToResponse(
                        nzyme,
                        organizationId,
                        tenantId,
                        L4Type.valueOf(negotiation.transport().toUpperCase()), ma))
                .toList();


        List<L4AddressResponse> relayedAddresses = negotiation.relayedAddresses()
                .stream()
                .map(ma -> RestHelpers.L4AddressDataToResponse(
                        nzyme,
                        organizationId,
                        tenantId,
                        L4Type.valueOf(negotiation.transport().toUpperCase()), ma))
                .toList();

        return NATSTUNNegotiationDetailsResponse.create(
                negotiation.negotiationKey(),
                negotiation.negotiationKeySha256(),
                negotiation.isActive(),
                negotiation.transport(),
                negotiation.successful(),
                negotiation.isTurn(),
                negotiation.bytesExchanged(),
                source,
                destination,
                mappedAddresses,
                peerAddresses,
                relayedAddresses,
                flows,
                negotiation.l4Tags(),
                relatedConnections,
                negotiation.firstSeen(),
                negotiation.lastActivity()
        );
    }

}
