
import React from 'react';
import UIStrings from '../../../../constants/UIStrings';

export default function RepositoryStatus({status}) {
  return <div>
    {status && status.online &&
      <span>{UIStrings.PAGES.REPOSITORY_STATUS.ONLINE}</span>
    }
    {!(status && status.online) &&
      <span>{UIStrings.PAGES.REPOSITORY_STATUS.OFFLINE}</span>
    }
    {status && status.description &&
      <span> - {status.description}</span>
    }
    {status && status.reason &&
      <div><i>{status.reason}</i></div>
    }
  </div>;
}                          
