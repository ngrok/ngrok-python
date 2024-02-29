from django.http import HttpResponse


async def home(request):
    response = HttpResponse("Hello")
    return response
