package org.globalart.astrum.repository.terraform.internal.browse;

import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Shows a module or provider version as the directory that holds its files, so a provider version with one
 * package per platform plus its checksums and signature is a single component node.
 */
@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
}
