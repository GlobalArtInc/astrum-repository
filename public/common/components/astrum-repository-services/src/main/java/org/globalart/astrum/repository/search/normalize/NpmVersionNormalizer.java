
package org.globalart.astrum.repository.search.normalize;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * npm's {@link SemVerVersionNormalizer} binding. See {@link SemVerVersionNormalizer} javadoc for why each
 * SemVer-compatible format needs its own trivial subclass rather than an aliasing {@code @Bean} method.
 */
@Component
@Qualifier("npm")
public class NpmVersionNormalizer
    extends SemVerVersionNormalizer
{
}
