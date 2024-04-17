using System;
using System.IO;
using System.Threading.Tasks;
using UnityEngine;
using GreClient.CardData;
using Newtonsoft.Json;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace mtga_datacollector
{
  public class MtgaDataDumper : MonoBehaviour
  {
    private string _dumpDirectory = String.Empty;
    private bool _dataDumped = false;

    public void InitializeDirectory(string directory)
    {
      _dumpDirectory = directory;
    }

    public void Start()
    {
      Task initialize = new Task(TryDump);
      initialize.Start();
    }

    private void TryDump()
    {
      DumpEverything();
      if (!_dataDumped)
      {
        System.Threading.Thread.Sleep(5000);
        TryDump();
      }
    }

    private void DumpEverything()
    {
      if (!_dataDumped && WrapperController.Instance != null && WrapperController.Instance.CardDatabase != null && WrapperController.Instance.CardDatabase.DatabaseUtilities != null && WrapperController.Instance.CardDatabase.CardTitleProvider != null)
      {
        try
        {
          string[] requiredSets = {"ANA", "ANB", "AHA1", "AHA2", "AHA3", "AHA4", "AHA5", "AHA6", "EA1", "XLN",
            "RIX", "DAR" /* this is "DOM" they just renamed it for some reason */, "M19", "GRN", "RNA", "WAR", "M20", "ELD", "THB", "IKO", "M21", "JMP",
            "AKR", "ZNR", "KLR", "KHM", "STX", "STA", "AFR", "MH1", "MH2", "J21", "MID", "Y22-MID" /* YMID in scry */,
            "VOW", "NEO", "Y22-NEO", "SNC", "Y22-SNC", "HBG", "DMU", "Y23-DMU" /* YDMU in scry */,
            "BRO", "Y23-BRO", "BRR", "EA2", "ONE", "Y23-ONE", "SIR", "SIS", "MOM", "MUL", "MAT", "LTR", "AHA7", "EA3", "WOE", "WOT", "Y24-WOE", "LCI", "Y24-LCI", "KTK", "MKM", "Y24-MKM", "OTJ", "SPG-MKM", "SPG-OTJ"
          };

          var setsWithCards = new Dictionary<string, List<CardPrintingData>>();
          foreach (var set in requiredSets)
          {
            setsWithCards.Add(set, new List<CardPrintingData>());
          }

          setsWithCards.Add("UNKNOWN", new List<CardPrintingData>());

          var db = WrapperController.Instance.CardDatabase;

          var allCards = WrapperController.Instance.CardDatabase.DatabaseUtilities.GetAllPrintings().Values;

          foreach (var card in allCards)
          {
            if (requiredSets.Contains(card.DigitalReleaseSet))
            {
              setsWithCards[card.DigitalReleaseSet].Add(card);
            }
            else if (requiredSets.Contains(card.ExpansionCode))
            {
              setsWithCards[card.ExpansionCode].Add(card);
            }
            else if (card.ExpansionCode == "G18")
            {
              setsWithCards["M19"].Add(card);
            }
            else
            {
              setsWithCards["UNKNOWN"].Add(card);
            }
          }

          List<MtgaPrintingData> every_card = new List<MtgaPrintingData>();
          HashSet<string> every_artist = new HashSet<string>();

          // MTGAPrintingData
          foreach (var set in setsWithCards)
          {
            List<MtgaPrintingData> exported_cards = new List<MtgaPrintingData>();
            exported_cards.Clear();
            foreach (var card in set.Value)
            {
              var cardData = new MtgaPrintingData
              {
                name = NormalizeName(db.CardTitleProvider.GetCardTitle(card.GrpId)),
                set = GetScrySetName(set.Key, card),
                arenaId = card.GrpId,
                isPrimaryCard = card.IsPrimaryCard,
                isMainSet = card.IsMainSet,
                isToken = card.IsToken,
                linkedFaces = card.LinkedFaceGrpIds.ToList(),
                isCollectible = CardUtilities.IsCardCollectible(card),
                isCraftable = CardUtilities.IsCardCraftable(card),
                tokens = card.AbilityIdToLinkedTokenGrpId.Values.SelectMany(x => x).ToList(),
                templates = card.LinkedAbilityTemplateCardGrpIds.ToList(),
                isRebalanced = card.IsRebalanced,
                rebalancedCardLink = card.RebalancedCardLink,
                artist = card.ArtistCredit,
                artId = card.ArtId,
                collectorNumber = NormalizeCollectorNumber(card, card.CollectorNumber),
                linkedFaceType = card.LinkedFaceType.ToString(),
                maxCollected = card.MaxCollected,
              };
              if (!string.IsNullOrWhiteSpace(cardData.name))
              {
                exported_cards.Add(cardData);
                every_card.Add(cardData);
              }

              every_artist.Add(card.ArtistCredit);

            }
            File.WriteAllText($"{_dumpDirectory}\\{set.Key}.txt", JsonConvert.SerializeObject(exported_cards.OrderBy(card => card.set).ThenBy(card => card.name).ThenBy(card => card.artist)));
          }

          StringBuilder artist_builder = new StringBuilder();
          foreach (var artist in every_artist)
          {
            artist_builder.AppendLine(artist);
          }

          File.WriteAllText($"{_dumpDirectory}\\artists.txt", artist_builder.ToString());
          File.WriteAllText($"{_dumpDirectory}\\dump.txt", JsonConvert.SerializeObject(every_card.OrderBy(card => card.set).ThenBy(card => card.name).ThenBy(card => card.artist)));
          _dataDumped = true;
        }
        catch (Exception e)
        {
          File.WriteAllText($"{_dumpDirectory}\\dump.txt", JsonConvert.SerializeObject(e));
        }
      }
    }

    private string NormalizeCollectorNumber(CardPrintingData card, string collectorNumber)
    {
      if (card.IsRebalanced && card.RebalancedCardLink != 0)
      {
        return $"A-{collectorNumber}";
      }
      else
      {
        return collectorNumber;
      }
    }

    private string NormalizeName(string v)
    {
      return v.Replace("<sprite=\"SpriteSheet_MiscIcons\" name=\"arena_a\">", "A-").Replace("<nobr>", "").Replace("</nobr>", "").Replace("<i>", "").Replace("</i>", "").Replace("///", "//");
    }

    private static string GetScrySetName(string arenaSetName, CardPrintingData printingData)
    {
      switch (arenaSetName)
      {
        case "UNKNOWN":
          {
            if (string.IsNullOrWhiteSpace(printingData.DigitalReleaseSet))
            {
              return $"{printingData.ExpansionCode}".ToLower();
            }
            else
            {
              return $"{printingData.DigitalReleaseSet}".ToLower();
            }
          }
        case "AHA1": return "ha1";
        case "AHA2": return "ha2";
        case "AHA3": return "ha3";
        case "AHA4": return "ha4";
        case "AHA5": return "ha5";
        case "AHA6": return "ha6";
        case "Y22-MID": return "ymid";
        case "Y22-SNC": return "ysnc";
        case "Y22-NEO": return "yneo";
        case "Y23-DMU": return "ydmu";
        case "DAR": return "dom";
        case "G18": return "m19"; // m19 gift card
        case "Y23-BRO": return "ybro";
        case "Y23-ONE": return "yone";
        case "Y24-WOE": return "ywoe";
        case "Y24-LCI": return "ylci";
        case "Y24-MKM": return "ymkm";
        case "AHA7": return "ha7";
        case "SPG-OTJ": return "spg";
        case "SPG-MKM": return "spg";
        default: return arenaSetName.ToLower();
      }
    }
  }
}