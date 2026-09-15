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

  const macFilter = (address, fieldName) => {
    if (!address) {
      return null;
    }

    return <FilterValueIcon setFilters={setFilters}
                            fields={WEBRTC_FILTER_FIELDS}
                            field={fieldName}
                            value={address.address} />
  }

  const columnSorting = (columnName) => {
    return <ColumnSorting thisColumn={columnName}
                          orderColumn={orderColumn}
                          setOrderColumn={setOrderColumn}
                          orderDirection={orderDirection}
                          setOrderDirection={setOrderDirection} />
  }

  const yes = () => {
    return <span className="text-success">Yes</span>
  }

  const no = () => {
    return <span className="text-muted">No</span>
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
        <tr>
          <th>&nbsp; {columnSorting("is_active")}</th>
          <th>ID</th>
          <th>Source MAC {columnSorting("source_mac")}</th>
          <th>Source Address {columnSorting("source_address")}</th>
          <th>Destination MAC {columnSorting("destination_mac")}</th>
          <th>Destination Address {columnSorting("destination_address")}</th>
          <th>Streams {columnSorting("stream_count")}</th>
          <th className="hide-narrow">RTP {columnSorting("has_rtp")}</th>
          <th className="hide-narrow">DTLS {columnSorting("has_dtls")}</th>
          <th className="hide-narrow">Audio {columnSorting("has_audio")}</th>
          <th className="hide-narrow">Video {columnSorting("has_video")}</th>
          <th>Bytes {columnSorting("bytes")}</th>
          <th>Duration {columnSorting("duration")}</th>
          <th>Initiated At {columnSorting("initiated_at")}</th>
          <th>Last Activity {columnSorting("last_activity")}</th>
        </tr>
        </thead>
        <tbody>
        {data.sessions.map((s, i) => {
          return (
            <tr key={i}>
              <td style={{width: 25}}>
                <WebRTCSessionActiveIndicator session={s} />
              </td>
              <td>
                <a href="">
                  <FullCopyShortenedId value={s.negotiation_key_sha256} />
                </a>
              </td>
              <td>
                <InternalAddressOnlyWrapper
                  address={s.source}
                  inner={s.source ? <EthernetMacAddress addressWithContext={s.source.mac}
                                                        filterElement={macFilter(s.source.mac, "source_mac")}
                                                        withAssetLink withAssetName /> : null} />
              </td>
              <td>
                <L4Address address={s.source}
                           hidePort={true}
                           filterElement={s.source ? <FilterValueIcon setFilters={setFilters}
                                                                      fields={WEBRTC_FILTER_FIELDS}
                                                                      field="source_address"
                                                                      value={s.source.address} /> : null } />
              </td>
              <td>
                <InternalAddressOnlyWrapper
                  address={s.destination}
                  inner={s.destination ? <EthernetMacAddress addressWithContext={s.destination.mac}
                                                             filterElement={macFilter(s.destination.mac, "destination_mac")}
                                                             withAssetLink withAssetName /> : null} />
              </td>
              <td>
                <L4Address address={s.destination}
                           hidePort={true}
                           filterElement={s.destination ? <FilterValueIcon setFilters={setFilters}
                                                                           fields={WEBRTC_FILTER_FIELDS}
                                                                           field="destination_address"
                                                                           value={s.destination.address} /> : null } />
              </td>
              <td>
                {numeral(s.stream_count).format("0,00")}

                <FilterValueIcon setFilters={setFilters}
                                 fields={WEBRTC_FILTER_FIELDS}
                                 field="stream_count"
                                 value={s.stream_count} />
              </td>
              <td className="hide-narrow">{s.has_rtp ? yes() : no()}</td>
              <td className="hide-narrow">{s.has_dtls ? yes() : no()}</td>
              <td className="hide-narrow">{s.has_audio ? yes() : no()}</td>
              <td className="hide-narrow">{s.has_video ? yes() : no()}</td>
              <td>
                {numeral(s.bytes_exchanged).format("0b")}

                <FilterValueIcon setFilters={setFilters}
                                 fields={WEBRTC_FILTER_FIELDS}
                                 field="bytes_exchanged"
                                 value={s.bytes_exchanged} />
              </td>
              <td>
                <FullCopy shortValue={formatDurationMs(s.duration_ms)} fullValue={s.duration_ms} />

                <FilterValueIcon setFilters={setFilters}
                                 fields={WEBRTC_FILTER_FIELDS}
                                 field="duration_ms"
                                 value={s.duration_ms} />
              </td>
              <td title={moment(s.first_seen).fromNow()}>
                {moment(s.first_seen).format()}
              </td>
              <td title={moment(s.last_activity).format()}>
                {moment(s.last_activity).fromNow()}
              </td>
            </tr>
          )
        })}
        </tbody>
      </table>

      <Paginator itemCount={data.total} perPage={perPage} setPage={setPage} page={page} />
    </React.Fragment>
  )

}