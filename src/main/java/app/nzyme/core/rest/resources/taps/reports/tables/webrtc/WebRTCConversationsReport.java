package app.nzyme.core.rest.resources.taps.reports.tables.webrtc;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.auto.value.AutoValue;

import java.util.List;

@AutoValue
public abstract class WebRTCConversationsReport {

    public abstract List<WebRTCConversationReport> conversations();

    @JsonCreator
    public static WebRTCConversationsReport create(@JsonProperty("conversations") List<WebRTCConversationReport> conversations) {
        return builder()
                .conversations(conversations)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_WebRTCConversationsReport.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder conversations(List<WebRTCConversationReport> conversations);

        public abstract WebRTCConversationsReport build();
    }
}
