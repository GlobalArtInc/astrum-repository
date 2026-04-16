
package org.globalart.astrum.repository.conda.internal.browse;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;

@Component
@Qualifier(CondaFormat.NAME)
public class CondaBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
}
