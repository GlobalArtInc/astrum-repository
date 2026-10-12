package org.globalart.astrum.repository.yum.internal.browse;

import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(YumFormat.NAME)
public class YumBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
}
