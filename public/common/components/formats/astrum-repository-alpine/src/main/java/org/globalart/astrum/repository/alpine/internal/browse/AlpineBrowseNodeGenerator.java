package org.globalart.astrum.repository.alpine.internal.browse;

import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
}
