
import React, {createContext, useContext} from 'react';
import {useInterpret, useActor} from '@xstate/react';
import RepositoriesListMachine from './RepositoriesListMachine';

const RepositoriesContext = createContext({});

export default function RepositoriesContextProvider(props) {
  const service = props.service ?? useInterpret(RepositoriesListMachine, {devTools: true});

  return (
    <RepositoriesContext.Provider value={{service}}>
      {props.children}
    </RepositoriesContext.Provider>
  );
};

export const useRepositoriesService = () => {
  const context = useContext(RepositoriesContext);
  return useActor(context.service);
}
