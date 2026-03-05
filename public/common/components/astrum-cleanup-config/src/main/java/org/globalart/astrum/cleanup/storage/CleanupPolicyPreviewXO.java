
package org.globalart.astrum.cleanup.storage;

public class CleanupPolicyPreviewXO
{
  private String repositoryName;

  private CleanupPolicyCriteria criteria;

  public String getRepositoryName() {
    return repositoryName;
  }

  public void setRepositoryName(final String repositoryName) {
    this.repositoryName = repositoryName;
  }

  public CleanupPolicyCriteria getCriteria() {
    return criteria;
  }

  public void setCriteria(final CleanupPolicyCriteria criteria) {
    this.criteria = criteria;
  }

  @Override
  public String toString() {
    return "CleanupPolicyPreviewXO(" +
        "repositoryName:" + repositoryName +
        ", criteria:" + criteria +
        ")";
  }
}
