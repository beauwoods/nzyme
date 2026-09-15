package app.nzyme.core.rest.responses.ethernet.webrtc;

import app.nzyme.core.rest.responses.ethernet.L4AddressResponse;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.auto.value.AutoValue;
import jakarta.annotation.Nullable;
import org.joda.time.DateTime;

import java.util.List;

@AutoValue
public abstract class WebRTCSessionDetailsResponse {

    @JsonProperty("negotiation_key")
    public abstract String negotiationKey();
    @JsonProperty("negotiation_key_sha256")
    public abstract String negotiationKeySha256();
    @JsonProperty("transport")
    public abstract String transport();
    @JsonProperty("is_active")
    public abstract boolean isActive();

    @JsonProperty("has_rtp")
    public abstract boolean hasRtp();
    @JsonProperty("has_dtls")
    public abstract boolean hasDtls();
    @JsonProperty("has_audio")
    public abstract boolean hasAudio();
    @JsonProperty("has_video")
    public abstract boolean hasVideo();
    @JsonProperty("stream_count")
    public abstract long streamCount();
    @JsonProperty("dtls_app_data_records")
    public abstract long dtlsAppDataRecords();
    @Nullable @JsonProperty("bytes_exchanged")
    public abstract Long bytesExchanged();
    @Nullable @JsonProperty("source")
    public abstract L4AddressResponse source();
    @Nullable @JsonProperty("destination")
    public abstract L4AddressResponse destination();
    @JsonProperty("rtp_streams")
    public abstract List<WebRTCRTPStreamDetailsResponse> rtpStreams();
    @JsonProperty("duration_ms")
    public abstract long durationMs();
    @JsonProperty("first_seen")
    public abstract DateTime firstSeen();
    @JsonProperty("last_activity")
    public abstract DateTime lastActivity();

    public static WebRTCSessionDetailsResponse create(String negotiationKey, String negotiationKeySha256, String transport, boolean isActive, boolean hasRtp, boolean hasDtls, boolean hasAudio, boolean hasVideo, long streamCount, long dtlsAppDataRecords, Long bytesExchanged, L4AddressResponse source, L4AddressResponse destination, List<WebRTCRTPStreamDetailsResponse> rtpStreams, long durationMs, DateTime firstSeen, DateTime lastActivity) {
        return builder()
                .negotiationKey(negotiationKey)
                .negotiationKeySha256(negotiationKeySha256)
                .transport(transport)
                .isActive(isActive)
                .hasRtp(hasRtp)
                .hasDtls(hasDtls)
                .hasAudio(hasAudio)
                .hasVideo(hasVideo)
                .streamCount(streamCount)
                .dtlsAppDataRecords(dtlsAppDataRecords)
                .bytesExchanged(bytesExchanged)
                .source(source)
                .destination(destination)
                .rtpStreams(rtpStreams)
                .durationMs(durationMs)
                .firstSeen(firstSeen)
                .lastActivity(lastActivity)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_WebRTCSessionDetailsResponse.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder negotiationKey(String negotiationKey);

        public abstract Builder negotiationKeySha256(String negotiationKeySha256);

        public abstract Builder transport(String transport);

        public abstract Builder isActive(boolean isActive);

        public abstract Builder hasRtp(boolean hasRtp);

        public abstract Builder hasDtls(boolean hasDtls);

        public abstract Builder hasAudio(boolean hasAudio);

        public abstract Builder hasVideo(boolean hasVideo);

        public abstract Builder streamCount(long streamCount);

        public abstract Builder dtlsAppDataRecords(long dtlsAppDataRecords);

        public abstract Builder bytesExchanged(Long bytesExchanged);

        public abstract Builder source(L4AddressResponse source);

        public abstract Builder destination(L4AddressResponse destination);

        public abstract Builder rtpStreams(List<WebRTCRTPStreamDetailsResponse> rtpStreams);

        public abstract Builder durationMs(long durationMs);

        public abstract Builder firstSeen(DateTime firstSeen);

        public abstract Builder lastActivity(DateTime lastActivity);

        public abstract WebRTCSessionDetailsResponse build();
    }
}
