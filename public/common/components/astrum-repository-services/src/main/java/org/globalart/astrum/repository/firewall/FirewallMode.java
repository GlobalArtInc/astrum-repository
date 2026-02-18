
package org.globalart.astrum.repository.firewall;

/**
 * Defines the operational modes for Astrum Firewall security scanning.
 * <p>
 * The firewall mode determines how the system responds to components that violate security policies
 * or fail policy evaluation checks.
 */
public enum FirewallMode
{
  /**
   * Disabled mode - firewall scanning is turned off for this repository.
   */
  DISABLED,

  /**
   * Audit mode - logs policy violations and security findings without blocking component downloads.
   * <p>
   * In this mode, the firewall records all security violations and policy failures but allows components
   * to be downloaded and used. This is useful for monitoring and reporting on security issues without
   * disrupting development workflows.
   */
  AUDIT,

  /**
   * Quarantine mode - blocks downloads of components that violate security policies.
   * <p>
   * In this mode, components that fail policy evaluation are prevented from being downloaded.
   * This enforces security policies by ensuring that only compliant components can be consumed
   * from the repository.
   */
  QUARANTINE,

  /**
   * PCCS mode - blocks downloads of components that fail pre-certified component scan checks.
   * <p>
   * Pre-Certified Component Scan (PCCS) mode specifically blocks components that have not been
   * pre-certified or have known security vulnerabilities identified through the PCCS process.
   * This is a specialized mode that integrates with GlobalArt's pre-certification services.
   */
  PCCS
}
