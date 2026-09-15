package app.nzyme.core.rest.resources.taps.reports.tables.webrtc;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.auto.value.AutoValue;

import java.util.List;

@AutoValue
public abstract class WebRTCRTPStreamReport {

    @JsonProperty("ssrc")
    public abstract long ssrc();
    @JsonProperty("direction")
    public abstract String direction();
    @JsonProperty("payload_types")
    public abstract List<Integer> payloadTypes();
    @JsonProperty("packet_count")
    public abstract long packetCount();
    @JsonProperty("byte_count")
    public abstract long byteCount();
    @JsonProperty("media_kind")
    public abstract String mediaKind();
    @JsonProperty("estimated_lost")
    public abstract int estimatedLost();
    @JsonProperty("sequence_min")
    public abstract int sequenceMin();
    @JsonProperty("sequence_max")
    public abstract int sequenceMax();
    @JsonProperty("timestamp_span")
    public abstract int timestampSpan();
    @JsonProperty("marker_count")
    public abstract long markerCount();

    @JsonCreator
    public static WebRTCRTPStreamReport create(@JsonProperty("ssrc") long ssrc,
                                               @JsonProperty("direction") String direction,
                                               @JsonProperty("payload_types") List<Integer> payloadTypes,
                                               @JsonProperty("packet_count") long packetCount,
                                               @JsonProperty("byte_count") long byteCount,
                                               @JsonProperty("media_kind") String mediaKind,
                                               @JsonProperty("estimated_lost") int estimatedLost,
                                               @JsonProperty("sequence_min") int sequenceMin,
                                               @JsonProperty("sequence_max") int sequenceMax,
                                               @JsonProperty("timestamp_span") int timestampSpan,
                                               @JsonProperty("marker_count") long markerCount) {
        return builder()
                .ssrc(ssrc)
                .direction(direction)
                .payloadTypes(payloadTypes)
                .packetCount(packetCount)
                .byteCount(byteCount)
                .mediaKind(mediaKind)
                .estimatedLost(estimatedLost)
                .sequenceMin(sequenceMin)
                .sequenceMax(sequenceMax)
                .timestampSpan(timestampSpan)
                .markerCount(markerCount)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_WebRTCRTPStreamReport.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder ssrc(long ssrc);

        public abstract Builder direction(String direction);

        public abstract Builder payloadTypes(List<Integer> payloadTypes);

        public abstract Builder packetCount(long packetCount);

        public abstract Builder byteCount(long byteCount);

        public abstract Builder mediaKind(String mediaKind);

        public abstract Builder estimatedLost(int estimatedLost);

        public abstract Builder sequenceMin(int sequenceMin);

        public abstract Builder sequenceMax(int sequenceMax);

        public abstract Builder timestampSpan(int timestampSpan);

        public abstract Builder markerCount(long markerCount);

        public abstract WebRTCRTPStreamReport build();
    }
}
