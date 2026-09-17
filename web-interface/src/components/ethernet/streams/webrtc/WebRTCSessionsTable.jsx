import React, {useContext, useEffect, useState} from "react";
import {TapContext} from "../../../../App";
import GenericWidgetLoadingSpinner from "../../../widgets/GenericWidgetLoadingSpinner";
import Paginator from "../../../misc/Paginator";
import useSelectedTenant from "../../../system/tenantselector/useSelectedTenant";
import ColumnSorting from "../../../shared/ColumnSorting";
import numeral from "numeral";
import FilterValueIcon from "../../../shared/filtering/FilterValueIcon";
import WebRTCService from "../../../../services/ethernet/WebRTCService";
import {WEBRTC_FILTER_FIELDS} from "./WebRTCFilterFields";
import WebRTCSessionActiveIndicator from "./WebRTCSessionActiveIndicator";
import FullCopyShortenedId from "../../../shared/FullCopyShortenedId";
import InternalAddressOnlyWrapper from "../../shared/InternalAddressOnlyWrapper";
import EthernetMacAddress from "../../../shared/context/macs/EthernetMacAddress";
import L4Address from "../../shared/L4Address";
import moment from "moment";
import FullCopy from "../../../shared/FullCopy";
import {formatDurationMs} from "../../../../util/Tools";
import WebRTCSessionsTableRow from "./WebRTCSessionsTableRow";
import WebRTCSessionsTableHead from "./WebRTCSessionsTableHead";

const webRTCService = new WebRTCService();

export default function WebRTCSessionsTable(props) {

  const [organizationId, tenantId] = useSelectedTenant();

  const timeRange = props.timeRange;
  const filters = props.filters;
  const setFilters = props.setFilters;
  const revision = props.revision;

  const [orderColumn, setOrderColumn] = useState("initiated_at");
  const [orderDirection, setOrderDirection] = useState("DESC");

  const [data, setData] = useState(null);

  const tapContext = useContext(TapContext);
  const selectedTaps = tapContext.taps;

  const perPage = props.perPage ? props.perPage : 25;
  const [page, setPage] = useState(1);

  useEffect(() => {
    setData(null);
    webRTCService.findAllSessions(organizationId, tenantId, timeRange, filters, orderColumn, orderDirection, selectedTaps, perPage, (page-1)*perPage, setData);
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

  if (data.sessions.length === 0) {
    return <div className="mb-0 alert alert-info">No WebRTC sessions were observed during selected time range.</div>
  }

  return (
    <React.Fragment>
      <strong>Total:</strong> {numeral(data.total).format("0,0")}

      <table className="table table-sm table-hover table-striped mb-4 mt-3">
        <thead>
          <WebRTCSessionsTableHead columnSorting={columnSorting} />
        </thead>
        <tbody>
        {data.sessions.map((s, i) => {
          return <WebRTCSessionsTableRow session={s} setFilters={setFilters} key={i} />
        })}
        </tbody>
      </table>

      <Paginator itemCount={data.total} perPage={perPage} setPage={setPage} page={page} />
    </React.Fragment>
  )

}