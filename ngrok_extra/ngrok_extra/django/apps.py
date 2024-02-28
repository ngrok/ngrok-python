from django.apps import AppConfig
from django.utils.translation import gettext_lazy as _


class NgrokConfig(AppConfig):
    name = "ngrok_extra.django"
    verbose_name = _("ngrok")

    def ready(self) -> None:
        return super().ready()
