package app.nzyme.core.database.generic;


import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class AssetPairNumberAggregationResultMapper implements RowMapper<AssetPairNumberAggregationResult> {
    @Override
    public AssetPairNumberAggregationResult map(ResultSet rs, StatementContext ctx) throws SQLException {
        return AssetPairNumberAggregationResult.create(
                rs.getString("value1"),
                rs.getString("value2"),
                rs.getLong("value3")
        );
    }
}
