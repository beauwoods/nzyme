import React from "react";

export default function WebRTCSessionActiveIndicator({session}) {

  if (session.is_active === null || session.is_active === undefined) {
    return <i className="fa fa-circle text-muted" title="Could not determine if session is active or not" />
  }

  if (session.is_active) {
    return <i className="fa fa-circle text-success blink" title="session is active" />
  } else {
    return <i className="fa fa-circle text-muted" title="session is not active" />
  }

}