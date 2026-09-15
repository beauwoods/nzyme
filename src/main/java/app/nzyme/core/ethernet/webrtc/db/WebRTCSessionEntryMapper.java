package app.nzyme.core.ethernet.webrtc.db;

import app.nzyme.core.ethernet.L4MapperTools;
import app.nzyme.core.ethernet.l4.db.L4AddressData;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;
import org.joda.time.DateTime;

import java.sql.ResultSet;
import java.sql.SQLException;

public class WebRTCSessionEntryMapper implements RowMapper<WebRTCSessionEntry> {

    @Override
    public WebRTCSessionEntry map(ResultSet rs, StatementContext ctx) throws SQLException {
        L4AddressData source = rs.getString("source_address") == null
                ? null : L4MapperTools.fieldsToAddressData("source", rs);
        L4AddressData destination = rs.getString("destination_address") == null
                ? null : L4MapperTools.fieldsToAddressData("destination", rs);

        String rtpStreams = rs.getString("rtp_streams");
        if (rtpStreams == null) {
            rtpStreams = "[]";
        }

        Long bytesExchanged = rs.getObject("bytes_exchanged") == null
                ? null : rs.getLong("bytes_exchanged");

        return WebRTCSessionEntry.create(
                rs.getString("negotiation_key"),
                rs.getString("negotiation_key_sha256"),
                rs.getString("transport"),
                rs.getBoolean("is_active"),
                rs.getBoolean("has_rtp"),
                rs.getBoolean("has_dtls"),
                rs.getBoolean("has_audio"),
                rs.getBoolean("has_video"),
                rs.getLong("stream_count"),
                rs.getLong("dtls_app_data_records"),
                bytesExchanged,
                source,
                destination,
                rtpStreams,
                rs.getLong("duration_ms"),
                new DateTime(rs.getTimestamp("first_seen")),
                new DateTime(rs.getTimestamp("last_activity"))
        );
    }
}