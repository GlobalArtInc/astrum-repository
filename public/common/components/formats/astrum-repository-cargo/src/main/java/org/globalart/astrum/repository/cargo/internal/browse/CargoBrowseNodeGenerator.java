package org.globalart.astrum.repository.cargo.internal.browse;

import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.content.browse.ComponentPathBrowseNodeGenerator;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CargoFormat.NAME)
public class CargoBrowseNodeGenerator
    extends ComponentPathBrowseNodeGenerator
{
}
