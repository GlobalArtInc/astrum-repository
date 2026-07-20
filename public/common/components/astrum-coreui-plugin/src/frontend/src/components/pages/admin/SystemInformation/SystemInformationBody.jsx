
import React from 'react';

import SystemInformationSection from "./SystemInformationSection";
import NestedSystemInformationSection from "./NestedSystemInformationSection";

/**
 * @since 3.24
 * @param systemInformation - a map with objects representing each section of system information
 */
export default function SystemInformationBody({systemInformation}) {
  return <>
    <SystemInformationSection
        sectionName="astrum-status"
        information={systemInformation['astrum-status']}
    />
    <SystemInformationSection
        sectionName="astrum-node"
        information={systemInformation['astrum-node']}
    />
    <SystemInformationSection
        sectionName="astrum-configuration"
        information={systemInformation['astrum-configuration']}
    />
    <SystemInformationSection
        sectionName="astrum-properties"
        information={systemInformation['astrum-properties']}
    />
    <SystemInformationSection
        sectionName="astrum-license"
        information={systemInformation['astrum-license']}
    />
    <SystemInformationSection
        sectionName="system-time"
        information={systemInformation['system-time']}
    />
    <SystemInformationSection
        sectionName="system-properties"
        information={systemInformation['system-properties']}
    />
    <SystemInformationSection
        sectionName="system-environment"
        information={systemInformation['system-environment']}
    />
    <SystemInformationSection
        sectionName="system-runtime"
        information={systemInformation['system-runtime']}
    />
    <NestedSystemInformationSection
        sectionName="system-network"
        sectionInformation={systemInformation['system-network']}
    />
    <NestedSystemInformationSection
        sectionName="system-filestores"
        sectionInformation={systemInformation['system-filestores']}
    />
  </>;
}
