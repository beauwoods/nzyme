package app.nzyme.core.rest.responses.ethernet.webrtc;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.auto.value.AutoValue;

import java.util.List;

@AutoValue
public abstract class WebRTCSessionsListResponse {

    @JsonProperty("total")
    public abstract long total();

    @JsonProperty("sessions")
    public abstract List<WebRTCSessionDetailsResponse> sessions();

    public static WebRTCSessionsListResponse create(long total, List<WebRTCSessionDetailsResponse> sessions) {
        return builder()
                .total(total)
                .sessions(sessions)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_WebRTCSessionsListResponse.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder total(long total);

        public abstract Builder sessions(List<WebRTCSessionDetailsResponse> sessions);

        public abstract WebRTCSessionsListResponse build();
    }
}
