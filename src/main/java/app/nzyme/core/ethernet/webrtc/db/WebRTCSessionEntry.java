package app.nzyme.core.ethernet.webrtc.db;

import app.nzyme.core.ethernet.l4.db.L4AddressData;
import com.google.auto.value.AutoValue;
import jakarta.annotation.Nullable;
import org.joda.time.DateTime;

@AutoValue
public abstract class WebRTCSessionEntry {

    public abstract String negotiationKey();
    public abstract String negotiationKeySha256();
    public abstract String transport();
    public abstract boolean isActive();

    public abstract boolean hasRtp();
    public abstract boolean hasDtls();
    public abstract boolean hasAudio();
    public abstract boolean hasVideo();
    public abstract long streamCount();
    public abstract long dtlsAppDataRecords();
    @Nullable
    public abstract Long bytesExchanged();
    @Nullable
    public abstract L4AddressData source();
    @Nullable
    public abstract L4AddressData destination();
    public abstract String rtpStreams();
    public abstract long durationMs();
    public abstract DateTime firstSeen();
    public abstract DateTime lastActivity();

    public static WebRTCSessionEntry create(String negotiationKey, String negotiationKeySha256, String transport, boolean isActive, boolean hasRtp, boolean hasDtls, boolean hasAudio, boolean hasVideo, long streamCount, long dtlsAppDataRecords, Long bytesExchanged, L4AddressData source, L4AddressData destination, String rtpStreams, long durationMs, DateTime firstSeen, DateTime lastActivity) {
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
        return new AutoValue_WebRTCSessionEntry.Builder();
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

        public abstract Builder source(L4AddressData source);

        public abstract Builder destination(L4AddressData destination);

        public abstract Builder rtpStreams(String rtpStreams);

        public abstract Builder durationMs(long durationMs);

        public abstract Builder firstSeen(DateTime firstSeen);

        public abstract Builder lastActivity(DateTime lastActivity);

        public abstract WebRTCSessionEntry build();
    }
}
