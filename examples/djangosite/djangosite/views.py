from django.http import HttpResponse


async def home(request):
    response = HttpResponse("Hello")
    return response


async def routeparams(request, year):
    response = HttpResponse(f"Year: {year}")
    return response


async def regex_test(request, number):
    response = HttpResponse(f"Number: {number}")
    return response


async def report(request, id=None):
    response = HttpResponse(f"Report: {id}")
    return response


async def creditadmin(request):
    response = HttpResponse(f"Credit Admin")
    return response
