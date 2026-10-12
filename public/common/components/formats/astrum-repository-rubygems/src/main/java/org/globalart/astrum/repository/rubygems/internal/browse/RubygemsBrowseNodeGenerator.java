package org.globalart.astrum.repository.rubygems.internal.browse;

import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
}
