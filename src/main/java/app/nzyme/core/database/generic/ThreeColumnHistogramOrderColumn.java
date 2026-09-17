package app.nzyme.core.database.generic;

public enum ThreeColumnHistogramOrderColumn {

    VALUE1("value1"),
    VALUE2("value2"),
    VALUE3("value3");

    private final String columnName;

    ThreeColumnHistogramOrderColumn(String columnName) {
        this.columnName = columnName;
    }

    public String getColumnName() {
        return columnName;
    }

}
