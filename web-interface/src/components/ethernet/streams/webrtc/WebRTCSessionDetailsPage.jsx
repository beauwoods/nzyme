import React, {useContext, useEffect, useState} from "react";
import usePageTitle from "../../../../util/UsePageTitle";
import {useParams} from "react-router-dom";
import {TapContext} from "../../../../App";
import useSelectedTenant from "../../../system/tenantselector/useSelectedTenant";
import {disableTapSelector, enableTapSelector} from "../../../misc/TapSelector";
import WebRTCService from "../../../../services/ethernet/WebRTCService";
import LoadingSpinner from "../../../misc/LoadingSpinner";
import ApiRoutes from "../../../../util/ApiRoutes";
import FullCopyShortenedId from "../../../shared/FullCopyShortenedId";
import CardTitleWithControls from "../../../shared/CardTitleWithControls";
import WebRTCSessionActiveIndicator from "./WebRTCSessionActiveIndicator";
import numeral from "numeral";
import L4Address from "../../shared/L4Address";
import InternalAddressOnlyWrapper from "../../shared/InternalAddressOnlyWrapper";
import EthernetMacAddress from "../../../shared/context/macs/EthernetMacAddress";
import L4SessionTags from "../../l4/L4SessionTags";
import moment from "moment/moment";

const webRTCService = new WebRTCService();

export default function WebRTCSessionDetailsPage() {

  usePageTitle("WebRTC Session Details");

  const { negotiationKey } = useParams();

  const tapContext = useContext(TapContext);
  const selectedTaps = tapContext.taps;

  const [organizationId, tenantId] = useSelectedTenant();

  const [session, setSession] = useState(null);

  useEffect(() => {
    enableTapSelector(tapContext);

    return () => {
      disableTapSelector(tapContext);
    }
  }, [tapContext]);

  useEffect(() => {
    setSession(null);
    webRTCService.findOneSessions(negotiationKey, organizationId, tenantId, selectedTaps, setSession);
  }, [negotiationKey, organizationId, tenantId, selectedTaps])

  const content = (session) => {
    if (!session.has_rtp && !session.has_dtls && !session.has_audio && !session.has_video) {
      return <span className="text-muted">n/a</span>
    }

    let contentTypes = [];

    if (session.has_rtp) { contentTypes.push("RTP"); }
    if (session.has_dtls) { contentTypes.push("DTLS"); }
    if (session.has_audio) { contentTypes.push("Audio"); }
    if (session.has_video) { contentTypes.push("Video"); }

    return contentTypes.join(", ")
  }

  if (session == null) {
    return <LoadingSpinner />
  }

  return (
    <React.Fragment>
      <div className="row">
        <div className="col-10">
          <nav aria-label="breadcrumb">
            <ol className="breadcrumb">
              <li className="breadcrumb-item"><a href={ApiRoutes.ETHERNET.STREAMS.WEBRTC.INDEX}>WebRTC Streams</a></li>
              <li className="breadcrumb-item">Sessions</li>
              <li className="breadcrumb-item active" aria-current="page">{negotiationKey}</li>
            </ol>
          </nav>
        </div>
        <div className="col-2">
          <a href={ApiRoutes.ETHERNET.STREAMS.WEBRTC.INDEX} className="btn btn-primary float-end">
            Back
          </a>
        </div>
      </div>

      <div className="row mt-3">
        <div className="col-12">
          <h1>
            WebRTC Session {<FullCopyShortenedId value={negotiationKey} />}
          </h1>
        </div>
      </div>

      <div className="row mt-3">
        <div className="col-4">
          <div className="card">
            <div className="card-body">
              <CardTitleWithControls title="Details" />

              <dl className="mb-0">
                <dt>Negotiation Key</dt>
                <dd className="machine-data">{session.negotiation_key}</dd>
                <dt>Transport</dt>
                <dd>{session.transport}</dd>
                <dt>Is Active</dt>
                <dd><WebRTCSessionActiveIndicator session={session} withText={true} /></dd>
                <dt>Tags</dt>
                <dd><L4SessionTags tags={session.tags} /></dd>
                <dt>RTP Streams</dt>
                <dd>{numeral(session.stream_count).format("0,0")}</dd>
                <dt>Sub-Sessions</dt>
                <dd>{session.sub_sessions ? numeral(session.sub_sessions.length).format("0,0") : "0"}</dd>
                <dt>Content</dt>
                <dd>
                  {content(session)}
                </dd>
              </dl>
            </div>
          </div>
        </div>

        <div className="col-4">
          <div className="card">
            <div className="card-body">
              <CardTitleWithControls title="Source &amp; Destination" />

              <dl className="mb-0">
                <dt>Bytes Exchanged</dt>
                <dd>{numeral(session.bytes_exchanged).format("0b")}</dd>
                <dt>Peer A Address</dt>
                <dd><L4Address address={session.source} hidePort={true}/></dd>
                <dt>Peer A Asset</dt>
                <dd>
                  <InternalAddressOnlyWrapper
                    address={session.source}
                    inner={session.source ?
                      <EthernetMacAddress addressWithContext={session.source.mac} withAssetLink withAssetName />
                      : null } />
                </dd>
                <dt>Peer B Address</dt>
                <dd><L4Address address={session.destination} hidePort={true}/></dd>
                <dt>Peer B Asset</dt>
                <dd>
                  <InternalAddressOnlyWrapper
                    address={session.destination}
                    inner={session.destination ?
                      <EthernetMacAddress addressWithContext={session.destination.mac} withAssetLink withAssetName />
                      : null } />
                </dd>
              </dl>
            </div>
          </div>
        </div>

        <div className="col-4">
          <div className="card">
            <div className="card-body">
              <CardTitleWithControls title="Metadata" />

              <dl className="mb-0">
                <dt>Initiated At</dt>
                <dd>
                  {moment(session.first_seen).format()} ({moment(session.first_seen).fromNow()})
                </dd>
                <dt>Last Activity</dt>
                <dd>
                  {moment(session.last_activity).format()} ({moment(session.last_activity).fromNow()})
                </dd>
                <dt>Duration</dt>
                <dd>
                  {moment.duration(
                    moment(session.last_activity).diff(moment(session.first_seen))
                  ).humanize()}
                </dd>
              </dl>
            </div>
          </div>
        </div>
      </div>
    </React.Fragment>
  )

}