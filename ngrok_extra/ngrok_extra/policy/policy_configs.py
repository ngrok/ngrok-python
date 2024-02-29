from dataclasses import dataclass
from typing import Dict, List


@dataclass
class LogConfig:
    metadata: Dict


@dataclass
class CustomResponseConfig:
    status_code: int
    content: str
    headers: Dict


@dataclass
class DenyConfig:
    status_code: int


@dataclass
class RateLimitConfig:
    name: str
    algorithm: str
    capacity: int
    rate: str
    bucket_key: Dict


@dataclass
class URLRewriteConfig:
    # These fields are different than the documentation. That is because "from" is a reserved keyword in Python.
    # When the policy is built, these will be remapped to the correct field names.
    match: str
    replacement: str

    def _as_json(self):
        return {"from": self.match, "to": self.replacement}


@dataclass
class AddHeadersConfig:
    headers: Dict


@dataclass
class RemoveHeadersConfig:
    headers: Dict


@dataclass
class JWTIssuerConfig:
    allow_list: List[Dict]


@dataclass
class JWTAudienceConfig:
    allow_list: List[Dict]


@dataclass
class JWTHttpToken:
    type: str
    method: str
    name: str
    prefix: str


@dataclass
class JWTHttpConfig:
    tokens: List[JWTHttpToken]


@dataclass
class JWTSigningKeySources:
    additional_jkus: List[str]


@dataclass
class JWTSigningKeys:
    sources: List[JWTSigningKeySources]


@dataclass
class JWTSigningConfig:
    allowed_algorithms: List[str]
    keys: JWTSigningKeys


@dataclass
class JWTValidationConfig:
    issuer: JWTIssuerConfig
    audience: JWTAudienceConfig
    http: JWTHttpConfig
    jwt: JWTSigningConfig
