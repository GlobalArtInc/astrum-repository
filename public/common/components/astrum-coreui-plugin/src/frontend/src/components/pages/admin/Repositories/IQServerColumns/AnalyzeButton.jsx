
import React, {useState} from 'react';

import {AsSegmentedButton, useToggle} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {HEALTH_CHECK} = UIStrings.REPOSITORIES.LIST;

const MODE = {
  THIS: 'this',
  ALL: 'all'
};

export default function AnalyzeButton({enableHealthCheck, name}) {
  const [mode, setMode] = useState(MODE.THIS);
  const [isOpen, onToggleOpen] = useToggle(false);

  const analyze = () => {
    const repoName = mode === MODE.THIS ? name : null;
    enableHealthCheck(repoName);
  };

  const selectAnalyzeThisRepo = () => {
    setMode(MODE.THIS);
    onToggleOpen();
  };

  const selectAnalyzeAllRepos = () => {
    setMode(MODE.ALL);
    onToggleOpen();
  };

  return (
    <AsSegmentedButton
      variant="primary"
      isOpen={isOpen}
      onToggleOpen={onToggleOpen}
      onClick={analyze}
      buttonContent={HEALTH_CHECK.ANALYZE_BUTTON}
    >
      <button className="as-dropdown-button" onClick={selectAnalyzeThisRepo}>
        {HEALTH_CHECK.ANALYZE_THIS(name)}
      </button>
      <button className="as-dropdown-button" onClick={selectAnalyzeAllRepos}>
        {HEALTH_CHECK.ANALYZE_ALL}
      </button>
    </AsSegmentedButton>
  );
}
