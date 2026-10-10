
import React from 'react';
import {useActor} from '@xstate/react';

import {
  AsFontAwesomeIcon,
  AsLoadWrapper,
  AsTextLink,
  AsTree
} from '@globalart/react-shared-components';

import {faFolder, faBox, faFileArchive} from '@fortawesome/free-solid-svg-icons';

const getIcon = (type) => {
  switch(type) {
    case 'folder':
      return faFolder 
    case 'component':
      return faBox
    default:
      return faFileArchive
  }
};

const BrowseTreeChildren = ({children}) => children?.map(child => {
  const [state, send] = useActor(child);
  const {children, loadError, node, open, repositoryName} = state.context;
  const isLoadingChildren = state.matches('chooseInitialState') || state.matches('loading');
  const baseUrl = '#browse/browse:' + encodeURIComponent(repositoryName);
  const encodedId = encodeURIComponent(node.id);

  return (
    <AsTree.Item key={node.id}
                 collapsible={!node.leaf}
                 isOpen={open}
                 onToggleCollapse={() => send({type: 'TOGGLE'})}
                 onActivate={() => document.getElementById(node.id).click()}>
      <AsTree.ItemLabel>
        <AsFontAwesomeIcon fixedWidth icon={getIcon(node.type)}/>
        <AsTextLink href={baseUrl + ':' + encodedId} id={node.id} tabIndex={-1}>{node.text}</AsTextLink>
      </AsTree.ItemLabel>
      <AsTree>
        <AsLoadWrapper retryHandler={() => send({type: 'RETRY'})} loading={isLoadingChildren} error={loadError}>
          {() => (children && <BrowseTreeChildren children={children}/>)}
        </AsLoadWrapper>
      </AsTree>
    </AsTree.Item>
  )
});

export default BrowseTreeChildren;
