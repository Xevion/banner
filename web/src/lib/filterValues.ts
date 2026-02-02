export const InstructionalMethodValues = {
  InPerson: "InPerson",
  Online: {
    Async: "Online.Async",
    Sync: "Online.Sync",
    Mixed: "Online.Mixed",
  },
  Hybrid: {
    Half: "Hybrid.Half",
    OneThird: "Hybrid.OneThird",
    TwoThirds: "Hybrid.TwoThirds",
  },
  Independent: "Independent",
} as const;

export const CampusValues = {
  Main: "Main",
  Downtown: "Downtown",
  Southwest: "Southwest",
  Laredo: "Laredo",
  Internet: "Internet",
  OnlinePrograms: "OnlinePrograms",
} as const;

export const PartOfTermValues = {
  FullTerm: "FullTerm",
  FirstHalf: "FirstHalf",
  SecondHalf: "SecondHalf",
} as const;

export const AttributeValues = {
  CoreCommunication: "CoreCommunication",
  CoreMathematics: "CoreMathematics",
  CoreLifePhysicalSciences: "CoreLifePhysicalSciences",
  CoreLanguagePhilosophy: "CoreLanguagePhilosophy",
  CoreCreativeArts: "CoreCreativeArts",
  CoreAmericanHistory: "CoreAmericanHistory",
  CoreGovernment: "CoreGovernment",
  CoreSocialBehavioral: "CoreSocialBehavioral",
  CoreComponentArea: "CoreComponentArea",
  Developmental: "Developmental",
  LowerDivision: "LowerDivision",
  UpperDivision: "UpperDivision",
  Graduate: "Graduate",
  Honors: "Honors",
  LowCostTextbooks: "LowCostTextbooks",
  FreeTextbooks: "FreeTextbooks",
  Leadership: "Leadership",
  ServiceLearning: "ServiceLearning",
  FinishAtUT: "FinishAtUT",
  UndergraduateResearch: "UndergraduateResearch",
} as const;
