package app.nzyme.core.rest.resources.taps.reports.tables.webrtc;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.auto.value.AutoValue;
import jakarta.annotation.Nullable;
import org.joda.time.DateTime;

import java.util.List;

@AutoValue
public abstract class WebRTCConversationReport {

    public abstract String negotiationKey();
    public abstract String sourceAddress();
    @Nullable
    public abstract String sourceMac();
    public abstract int sourcePort();
    public abstract String destinationAddress();
    @Nullable
    public abstract String destinationMac();
    public abstract int destinationPort();
    public abstract boolean hasRtp();
    public abstract boolean hasDtls();
    public abstract boolean hasAudio();
    public abstract boolean hasVideo();
    public abstract int streamCount();
    public abstract List<WebRTCRTPStreamReport> rtpStreams();
    public abstract long dtlsAppDataRecords();
    public abstract DateTime firstSeen();
    public abstract DateTime lastActivity();
    public abstract long bytesRx();
    public abstract long bytesTx();

    @JsonCreator
    public static WebRTCConversationReport create(@JsonProperty("negotiation_key") String negotiationKey,
                                                  @JsonProperty("source_address") String sourceAddress,
                                                  @JsonProperty("source_mac") String sourceMac,
                                                  @JsonProperty("source_port") int sourcePort,
                                                  @JsonProperty("destination_address")String destinationAddress,
                                                  @JsonProperty("destination_mac") String destinationMac,
                                                  @JsonProperty("destination_port") int destinationPort,
                                                  @JsonProperty("has_rtp") boolean hasRtp,
                                                  @JsonProperty("has_dtls") boolean hasDtls,
                                                  @JsonProperty("has_audio") boolean hasAudio,
                                                  @JsonProperty("has_video") boolean hasVideo,
                                                  @JsonProperty("stream_count") int streamCount,
                                                  @JsonProperty("rtp_streams") List<WebRTCRTPStreamReport> rtpStreams,
                                                  @JsonProperty("dtls_app_data_records") long dtlsAppDataRecords,
                                                  @JsonProperty("first_seen") DateTime firstSeen,
                                                  @JsonProperty("last_activity") DateTime lastActivity,
                                                  @JsonProperty("bytes_rx") long bytesRx,
                                                  @JsonProperty("bytes_tx") long bytesTx) {
        return builder()
                .negotiationKey(negotiationKey)
                .sourceAddress(sourceAddress)
                .sourceMac(sourceMac)
                .sourcePort(sourcePort)
                .destinationAddress(destinationAddress)
                .destinationMac(destinationMac)
                .destinationPort(destinationPort)
                .hasRtp(hasRtp)
                .hasDtls(hasDtls)
                .hasAudio(hasAudio)
                .hasVideo(hasVideo)
                .streamCount(streamCount)
                .rtpStreams(rtpStreams)
                .dtlsAppDataRecords(dtlsAppDataRecords)
                .firstSeen(firstSeen)
                .lastActivity(lastActivity)
                .bytesRx(bytesRx)
                .bytesTx(bytesTx)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_WebRTCConversationReport.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder negotiationKey(String negotiationKey);

        public abstract Builder sourceAddress(String sourceAddress);

        public abstract Builder sourceMac(String sourceMac);

        public abstract Builder sourcePort(int sourcePort);

        public abstract Builder destinationAddress(String destinationAddress);

        public abstract Builder destinationMac(String destinationMac);

        public abstract Builder destinationPort(int destinationPort);

        public abstract Builder hasRtp(boolean hasRtp);

        public abstract Builder hasDtls(boolean hasDtls);

        public abstract Builder hasAudio(boolean hasAudio);

        public abstract Builder hasVideo(boolean hasVideo);

        public abstract Builder streamCount(int streamCount);

        public abstract Builder rtpStreams(List<WebRTCRTPStreamReport> rtpStreams);

        public abstract Builder dtlsAppDataRecords(long dtlsAppDataRecords);

        public abstract Builder firstSeen(DateTime firstSeen);

        public abstract Builder lastActivity(DateTime lastActivity);

        public abstract Builder bytesRx(long bytesRx);

        public abstract Builder bytesTx(long bytesTx);

        public abstract WebRTCConversationReport build();
    }
}
