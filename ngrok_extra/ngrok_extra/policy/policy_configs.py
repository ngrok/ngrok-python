from dataclasses import dataclass

@dataclass
class LogConfig:
    metadata: dict

@dataclass
class CustomResponseConfig:
    status_code: int
    content: str
    headers: dict

@dataclass
class DenyConfig:
    status_code: int

@dataclass
class RateLimitConfig:
    name: str
    algorithm: str
    capacity: int
    rate: str
    bucket_key: dict

@dataclass
class URLRewriteConfig:
    # These fields are different than the documentation. That is because "from" is a reserved keyword in Python.
    # When the policy is built, these will be remapped to the correct field names.
    match: str
    replacement: str

    def _as_json(self):
        return {
            "from": self.match,
            "to": self.replacement
        }

@dataclass
class AddHeadersConfig:
    headers: dict

@dataclass
class RemoveHeadersConfig:
    headers: dict

@dataclass
class JWTIssuerConfig:
    allow_list: [dict]

@dataclass
class JWTAudienceConfig:
    allow_list: [dict]

@dataclass
class JWTHttpToken:
    type: str
    method: str
    name: str
    prefix: str
    
@dataclass
class JWTHttpConfig:
    tokens: [JWTHttpToken]
    
@dataclass
class JWTSigningKeySources:
    additional_jkus: [str]

@dataclass
class JWTSigningKeys:
    sources: [JWTSigningKeySources]
    
@dataclass
class JWTSigningConfig:
    allowed_algorithms: [str]
    keys: JWTSigningKeys

@dataclass
class JWTValidationConfig:
    issuer: JWTIssuerConfig
    audience: JWTAudienceConfig
    http: JWTHttpConfig
    jwt: JWTSigningConfig

