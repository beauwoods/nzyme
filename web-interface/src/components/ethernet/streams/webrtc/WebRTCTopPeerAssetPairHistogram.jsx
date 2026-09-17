import React, {useContext, useEffect, useState} from "react";
import {DEFAULT_LIMIT} from "../../../widgets/LimitSelector";
import WebRTCService from "../../../../services/ethernet/WebRTCService";
import LoadingSpinner from "../../../misc/LoadingSpinner";
import {TapContext} from "../../../../App";
import useSelectedTenant from "../../../system/tenantselector/useSelectedTenant";
import ThreeColumnHistogram from "../../../widgets/histograms/ThreeColumnHistogram";

const webRTCService = new WebRTCService();

export default function WebRTCTopPeerAssetPairHistogram({timeRange, filters, revision}) {

  const [organizationId, tenantId] = useSelectedTenant();

  const tapContext = useContext(TapContext);
  const selectedTaps = tapContext.taps;

  const [limit, setLimit] = useState(DEFAULT_LIMIT);
  const [histogram, setHistogram] = useState(null);

  const [orderColumn, setOrderColumn] = useState("value3");
  const [orderDirection, setOrderDirection] = useState("DESC");

  useEffect(() => {
    setHistogram(null);

    webRTCService.getTopPeerAssetPairHistogram(
      setHistogram, organizationId, tenantId, timeRange, orderColumn, orderDirection, limit, 0, filters, selectedTaps
    );
  }, [selectedTaps, organizationId, tenantId, limit, timeRange, filters, orderColumn, orderDirection, revision]);

  if (!histogram) {
    return <LoadingSpinner />
  }

  if (histogram.total === 0) {
    return (
      <div className="alert alert-info mb-0 mt-2">
        No WebRTC sessions recorded.
      </div>
    )
  }

  return <ThreeColumnHistogram data={histogram}
                               columnTitles={["Asset", "Asset", "Bytes Exchanged"]}
                               orderColumn={orderColumn}
                               setOrderColumn={setOrderColumn}
                               orderDirection={orderDirection}
                               setOrderDirection={setOrderDirection}
                               limit={limit}
                               setLimit={setLimit} />

}