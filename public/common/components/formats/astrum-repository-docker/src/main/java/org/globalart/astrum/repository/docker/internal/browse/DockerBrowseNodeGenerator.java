package org.globalart.astrum.repository.docker.internal.browse;

import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.docker.DockerFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
public class DockerBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
}
