package org.globalart.astrum.repository.nuget.internal.browse;

import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
}
