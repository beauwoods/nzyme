package app.nzyme.core.database.generic;

import app.nzyme.core.ethernet.l4.db.L4AddressData;
import com.google.auto.value.AutoValue;

@AutoValue
public abstract class AddressPairNumberAggregationResult {

    public abstract L4AddressData address1();
    public abstract L4AddressData address2();
    public abstract long value();

    public static AddressPairNumberAggregationResult create(L4AddressData address1, L4AddressData address2, long value) {
        return builder()
                .address1(address1)
                .address2(address2)
                .value(value)
                .build();
    }

    public static Builder builder() {
        return new AutoValue_AddressPairNumberAggregationResult.Builder();
    }

    @AutoValue.Builder
    public abstract static class Builder {
        public abstract Builder address1(L4AddressData address1);

        public abstract Builder address2(L4AddressData address2);

        public abstract Builder value(long value);

        public abstract AddressPairNumberAggregationResult build();
    }
}