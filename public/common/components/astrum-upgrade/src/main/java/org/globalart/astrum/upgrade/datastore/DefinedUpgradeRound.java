
package org.globalart.astrum.upgrade.datastore;

/**
 * An interface intended for use with {@link RepeatableDatabaseMigrationStep} which allows developers to specify which
 * round a step should be included in. This is primarily intended when an OSS step must be delayed until after a PRO
 * step.<br/>
 *
 * See also {@link SimpleDependencyResolver} for details of how rounds are computed.
 */
public interface DefinedUpgradeRound
{
  int getUpgradeRound();
}
