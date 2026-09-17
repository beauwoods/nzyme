package app.nzyme.core.database.generic;

import com.google.auto.value.AutoValue;

@AutoValue
public abstract class AssetPairNumberAggregationResult {

    public abstract String mac1();
    public abstract String mac2();
    public abstract long value();

    public static AssetPairNumberAggregationResult create(String mac1, String mac2, long value) {
        return builder()
                .mac1(mac1)
                .mac2(mac2)
                .value(value)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_AssetPairNumberAggregationResult.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder mac1(String mac1);

        public abstract Builder mac2(String mac2);

        public abstract Builder value(long value);

        public abstract AssetPairNumberAggregationResult build();
    }
}
