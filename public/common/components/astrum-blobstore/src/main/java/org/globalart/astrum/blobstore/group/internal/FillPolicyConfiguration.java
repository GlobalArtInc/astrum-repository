
package org.globalart.astrum.blobstore.group.internal;

import org.globalart.astrum.blobstore.group.FillPolicy;

import jakarta.inject.Provider;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FillPolicyConfiguration
{
  @Bean
  public Provider<FillPolicy> roundRobinFillPolicyProvider(final ObjectProvider<RoundRobinFillPolicy> provider) {
    return new RoundRobinFillPolicyProvider(provider);
  }

  @Bean
  public Provider<FillPolicy> writeToFirstFillPolicyProvider(
      final ObjectProvider<WriteToFirstMemberFillPolicy> provider)
  {
    return new WriteToFirstMemberFillPolicyProvider(provider);
  }

  @Qualifier(WriteToFirstMemberFillPolicy.TYPE)
  private record WriteToFirstMemberFillPolicyProvider(ObjectProvider<WriteToFirstMemberFillPolicy> provider)
      implements Provider<FillPolicy>
  {
    @Override
    public FillPolicy get() {
      return provider.getIfUnique();
    }
  }

  @Qualifier(RoundRobinFillPolicy.TYPE)
  private record RoundRobinFillPolicyProvider(ObjectProvider<RoundRobinFillPolicy> provider)
      implements Provider<FillPolicy>
  {
    @Override
    public FillPolicy get() {
      return provider.getIfUnique();
    }
  }
}
