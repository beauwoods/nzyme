import React, {useContext, useEffect, useState} from "react";
import NATService from "../../../../../services/ethernet/NATService";
import useSelectedTenant from "../../../../system/tenantselector/useSelectedTenant";
import {TapContext} from "../../../../../App";
import ColumnSorting from "../../../../shared/ColumnSorting";
import FilterValueIcon from "../../../../shared/filtering/FilterValueIcon";
import GenericWidgetLoadingSpinner from "../../../../widgets/GenericWidgetLoadingSpinner";
import {STUN_CONNECTIONS_FILTER_FIELDS} from "./STUNConnectionsFilterFields";
import numeral from "numeral";
import InternalAddressOnlyWrapper from "../../../shared/InternalAddressOnlyWrapper";
import EthernetMacAddress from "../../../../shared/context/macs/EthernetMacAddress";
import L4Address from "../../../shared/L4Address";
import moment from "moment/moment";
import Paginator from "../../../../misc/Paginator";
import FullCopyShortenedId from "../../../../shared/FullCopyShortenedId";
import ApiRoutes from "../../../../../util/ApiRoutes";
import STUNConnectionActiveIndicator from "./STUNConnectionActiveIndicator";
import STUNConnectionSuccessIndicator from "./STUNConnectionSuccessIndicator";
import STUNConnectionL4Tags from "./STUNConnectionL4Tags";
import STUNConnectionsTableHead from "./STUNConnectionsTableHead";
import STUNConnectionsTableRow from "./STUNConnectionsTableRow";

const natService = new NATService();

export default function STUNConnectionsTable({timeRange, filters, setFilters, revision, perPage}) {

  const [organizationId, tenantId] = useSelectedTenant();

  const [orderColumn, setOrderColumn] = useState("initiated_at");
  const [orderDirection, setOrderDirection] = useState("DESC");

  const [data, setData] = useState(null);

  const tapContext = useContext(TapContext);
  const selectedTaps = tapContext.taps;

  const perPageSel = perPage ? perPage : 25;
  const [page, setPage] = useState(1);

  useEffect(() => {
    setData(null);
    natService.findAllSTUNConnections(organizationId, tenantId, timeRange, filters, orderColumn, orderDirection, selectedTaps, perPageSel, (page-1)*perPageSel, setData);
  }, [organizationId, tenantId, selectedTaps, timeRange, filters, orderColumn, orderDirection, page, revision]);

  const columnSorting = (columnName) => {
    return <ColumnSorting thisColumn={columnName}
                          orderColumn={orderColumn}
                          setOrderColumn={setOrderColumn}
                          orderDirection={orderDirection}
                          setOrderDirection={setOrderDirection} />
  }

  if (!data) {
    return <GenericWidgetLoadingSpinner height={150} />
  }

  if (data.total === 0) {
    return <div className="mb-0 alert alert-info">No NAT connection attempts were observed during selected time range.</div>
  }

  return (
    <React.Fragment>
      <strong>Total:</strong> {numeral(data.total).format("0,0")}

      <table className="table table-sm table-hover table-striped mb-4 mt-3">
        <thead>
          <STUNConnectionsTableHead columnSorting={columnSorting} />
        </thead>
        <tbody>
        {data.negotiations.map((n, i) => {
          return <STUNConnectionsTableRow key={i} connection={n} setFilters={setFilters} />
        })}
        </tbody>
      </table>

      <Paginator itemCount={data.total} perPage={perPageSel} setPage={setPage} page={page} />
    </React.Fragment>
  )

}