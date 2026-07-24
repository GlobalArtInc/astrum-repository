
import React from 'react';

export default function RepositoryStatus({status}) {
  return <div>
    {status && status.online &&
      <span>Online</span>
    }
    {!(status && status.online) &&
      <span>Offline</span>
    }
    {status && status.description &&
      <span> - {status.description}</span>
    }
    {status && status.reason &&
      <div><i>{status.reason}</i></div>
    }
  </div>;
}                          
