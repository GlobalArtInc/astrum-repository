package org.globalart.astrum.repository.cargo.internal;

import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.search.normalize.SemVerVersionNormalizer;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CargoFormat.NAME)
public class CargoVersionNormalizer
    extends SemVerVersionNormalizer
{
}
