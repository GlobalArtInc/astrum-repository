package org.globalart.astrum.repository.cocoapods.internal.browse;

import org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CocoapodsFormat.NAME)
public class CocoapodsBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
}
