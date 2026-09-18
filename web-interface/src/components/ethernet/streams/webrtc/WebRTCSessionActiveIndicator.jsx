import React from "react";

export default function WebRTCSessionActiveIndicator({session, withText = false}) {

  if (session.is_active === null || session.is_active === undefined) {
    return (
      <>
        <i className="fa fa-circle text-muted" title="Could not determine if session is active or not" />
        {withText ? <span>&nbsp; Unknown</span> : null}
      </>
    )
  }

  if (session.is_active) {
    return (
      <>
        <i className="fa fa-circle text-success blink" title="session is active" />
        {withText ? <span>&nbsp; Active</span> : null}
      </>
    )
  } else {
    return (
      <>
        <i className="fa fa-circle text-muted" title="session is not active" />
        {withText ? <span>&nbsp; Inactive</span> : null}
      </>
    )
  }

}