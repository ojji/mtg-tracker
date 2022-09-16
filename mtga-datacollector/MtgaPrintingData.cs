using System.Collections.Generic;

namespace mtga_datacollector
{
  public class MtgaPrintingData
  {
    public string name;
    public string set;
    public uint arenaId;
    public bool isPrimaryCard;
    public bool isMainSet;
    public bool isToken;
    public List<uint> linkedFaces;
    public bool isCollectible;
    public bool isCraftable;
    public List<uint> tokens;
    public List<uint> templates;
    public bool isRebalanced;
    public uint rebalancedCardLink;
    public string artist;
    public uint artId;
    public string collectorNumber;
    public string linkedFaceType;
  }
}
