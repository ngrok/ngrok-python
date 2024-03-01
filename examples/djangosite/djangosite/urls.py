"""djangosite URL Configuration

The `urlpatterns` list routes URLs to views. For more information please see:
    https://docs.djangoproject.com/en/4.1/topics/http/urls/
Examples:
Function views
    1. Add an import:  from my_app import views
    2. Add a URL to urlpatterns:  path('', views.home, name='home')
Class-based views
    1. Add an import:  from other_app.views import Home
    2. Add a URL to urlpatterns:  path('', Home.as_view(), name='home')
Including another URLconf
    1. Import the include() function: from django.urls import include, path
    2. Add a URL to urlpatterns:  path('blog/', include('blog.urls'))
"""
from django.contrib import admin
from django.urls import include

# from django.urls import path, re_path
from ngrok_extra.django.urls import path, re_path
from ngrok_extra.policy import policy_builder

from .views import home, routeparams, regex_test, report, creditadmin

extra_patterns = [
    path(
        "admin/",
        creditadmin,
        # inbound_rule=policy_builder.PolicyRule().with_deny(
        #     policy_builder.DenyConfig(status_code=403)
        # ),
    ),
    path(
        "reports/",
        report,
        outbound_rule=policy_builder.PolicyRule().with_add_headers(
            policy_builder.AddHeadersConfig(
                headers={"added-header-urls": "nested, no route param"}
            )
        ),
    ),
    path(
        "reports/<int:id>/",
        report,
        outbound_rule=policy_builder.PolicyRule().with_add_headers(
            policy_builder.AddHeadersConfig(
                headers={"added-header-urls": "nested, with route param"}
            )
        ),
    ),
]

urlpatterns = [
    path(
        "",
        home,
        outbound_rule=policy_builder.PolicyRule().with_add_headers(
            policy_builder.AddHeadersConfig(headers={"added-header-urls": "home-page"})
        ),
    ),
    path(
        "year/<int:year>/",
        routeparams,
        outbound_rule=policy_builder.PolicyRule().with_add_headers(
            policy_builder.AddHeadersConfig(
                headers={"added-header-urls": "route-params"}
            )
        ),
    ),
    re_path(
        r"^regex/(?:test-(?P<number>[0-9]+)/)?$",
        regex_test,
        outbound_rule=policy_builder.PolicyRule().with_add_headers(
            policy_builder.AddHeadersConfig(headers={"added-header-urls": "regex"})
        ),
    ),
    path(
        "admin/",
        admin.site.urls,
        inbound_rule=policy_builder.PolicyRule().with_deny(
            policy_builder.DenyConfig(status_code=403)
        ),
    ),
    path(
        "credit/",
        include(extra_patterns),
        # inbound_rule=policy_builder.PolicyRule().with_deny(
        #     policy_builder.DenyConfig(status_code=403)
        # ),
    ),
]
