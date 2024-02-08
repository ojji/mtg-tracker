using System;
using System.Threading.Tasks;
using UnityEngine;
using Wizards.Mtga.FrontDoorModels;
using Newtonsoft.Json;
using System.Threading;
using System.Linq;
using Wizards.Models;

namespace mtga_datacollector
{
  public class CollectorEvent
  {
    public string Timestamp;
    public string Source;
    public object Attachment;
  }

  public class AccountData
  {
    public string userId;
    public string screenName;
  }

  public class CardInventoryData
  {
    public uint grpId;
    public int count;
  }

  public class MtgaDataCollector : MonoBehaviour
  {
    private UnityCrossThreadLogger _logger = new UnityCrossThreadLogger("MTGADataCollector");
    private bool _subscribedToAccountInfo = false;
    private bool _subscribedToInventory = false;

    public void Start()
    {
      _logger.Info($"[initialization]Initialization started at {System.DateTime.Now:O}");
      Task initialize = new Task(Initialize);
      initialize.Start();
    }

    private void Initialize()
    {
      if (!_subscribedToAccountInfo)
      {
        SubscribeToAccountInfo();
      }
      if (!_subscribedToInventory)
      {
        CollectInventoryAndSubscribeToChanges();
      }
      if (!_subscribedToInventory || !_subscribedToAccountInfo)
      {
        _logger.Info($"[initialization]Waiting for everyone to load {System.DateTime.Now:O}");
        System.Threading.Thread.Sleep(5000);
        Initialize();
      }

      _logger.Info($"[initialization]Initialization is done at {System.DateTime.Now:O}. Ready to go!");
    }

    private void SubscribeToAccountInfo()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        try
        {
          WrapperController.Instance.AccountClient.LoginStateChanged += AccountClientLoginStateChanged;
          var accountInfo = new AccountData
          {
            userId = WrapperController.Instance.AccountClient.AccountInformation.AccountID,
            screenName = WrapperController.Instance.AccountClient.AccountInformation.DisplayName
          };

          var logEntry = new CollectorEvent
          {
            Attachment = accountInfo,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          WriteToLog("account-info", logEntry);
          _subscribedToAccountInfo = true;
        }
        catch (Exception e)
        {
          _logger.Info($"[account-info-error]{JsonConvert.SerializeObject(e)}");
        }
      }
    }

    private void AccountClientLoginStateChanged(LoginState obj)
    {
      _logger.Info($"[loginstate]{obj}");
      _subscribedToAccountInfo = false;
      _subscribedToInventory = false;
      if (WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        WrapperController.Instance.AccountClient.LoginStateChanged -= AccountClientLoginStateChanged;
      }
      Task.Run(Initialize);
    }

    private void CollectInventoryAndSubscribeToChanges()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Cards.Count > 0)
      {
        try
        {
          var inventoryManager = WrapperController.Instance.InventoryManager;

          inventoryManager.Subscribe(InventoryUpdateSource.Unknown, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "Unknown"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.QuestReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "QuestReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.DailyWins, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "DailyWins"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.WeeklyWins, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "WeeklyWins"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.LoginGrant, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "LoginGrant"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.BattlePassLevelUp, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "BattlePassLevelUp"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.BattlePassLevelMasteryTree, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "BattlePassLevelMasteryTree"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EarlyPlayerProgressionLevelUp, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EarlyPlayerProgressionLevelUp"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EarlyPlayerProgressionMasteryTree, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EarlyPlayerProgressionMasteryTree"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.ProgressionRewardTierAdd, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "ProgressionRewardTierAdd"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.RenewalReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "RenewalReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.MercantilePurchase, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "MercantilePurchase"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.MercantileChestPurchase, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "MercantileChestPurchase"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.MercantileBoosterPurchase, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "MercantileBoosterPurchase"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EventReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EventReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.RedeemVoucher, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "RedeemVoucher"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.ModifyPlayerInventory, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "ModifyPlayerInventory"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.OpenChest, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "OpenChest"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.MassOpenChest, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "MassOpenChest"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.BasicLandSetUpdate, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "BasicLandSetUpdate"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CompleteVault, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CompleteVault"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CosmeticPurchase, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CosmeticPurchase"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.WildCardRedemption, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "WildCardRedemption"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.BoosterOpen, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "BoosterOpen"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.StarterDeckUpgrade, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "StarterDeckUpgrade"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.RankedSeasonReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "RankedSeasonReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EventPayEntry, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EventPayEntry"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.BannedCardGrant, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "BannedCardGrant"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EventEntryReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EventEntryReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CatalogPurchase, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CatalogPurchase"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CampaignGraphReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EventRefundEntry, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EventRefundEntry"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.Cleanup, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "Cleanup"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.IdEmpotentLoginGrant, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "IdEmpotentLoginGrant"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CustomerSupportGrant, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CustomerSupportGrant"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EntryReward, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EntryReward"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.EventGrantCardPool, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "EventGrantCardPool"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphPayoutNode, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CampaignGraphPayoutNode"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphAutomaticPayoutNode, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CampaignGraphAutomaticPayoutNode"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphPurchaseNode, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CampaignGraphPurchaseNode"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphTieredRewardNode, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "CampaignGraphTieredRewardNode"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.AccumulativePayoutNode, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "AccumulativePayoutNode"); }), publish: false);
          inventoryManager.Subscribe(InventoryUpdateSource.Letter, new Action<ClientInventoryUpdateReportItem>(item => { UpdateInventory(item, "Letter"); }), publish: false);

          /*
           *Unknown,
  QuestReward,
  DailyWins,
  WeeklyWins,
  LoginGrant,
  BattlePassLevelUp,
  BattlePassLevelMasteryTree,
  EarlyPlayerProgressionLevelUp,
  EarlyPlayerProgressionMasteryTree,
  ProgressionRewardTierAdd,
  RenewalReward,
  MercantilePurchase,
  MercantileChestPurchase,
  MercantileBoosterPurchase,
  EventReward,
  RedeemVoucher,
  ModifyPlayerInventory,
  OpenChest,
  MassOpenChest,
  BasicLandSetUpdate,
  CompleteVault,
  CosmeticPurchase,
  WildCardRedemption,
  BoosterOpen,
  StarterDeckUpgrade,
  RankedSeasonReward,
  EventPayEntry,
  BannedCardGrant,
  EventEntryReward,
  CatalogPurchase,
  CampaignGraphReward,
  EventRefundEntry,
  Cleanup,
  IdEmpotentLoginGrant,
  CustomerSupportGrant,
  EntryReward,
  EventGrantCardPool,
  CampaignGraphPayoutNode,
  CampaignGraphAutomaticPayoutNode,
  CampaignGraphPurchaseNode,
  CampaignGraphTieredRewardNode,
  AccumulativePayoutNode,
  Letter
           */

          //inventoryManager.Subscribe(InventoryUpdateSource.MercantilePurchase, new Action<ClientInventoryUpdateReportItem>(this.OnMercantilePurchase), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.MercantileChestPurchase, new Action<ClientInventoryUpdateReportItem>(this.OnMercantileChestPurchase), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.CatalogPurchase, new Action<ClientInventoryUpdateReportItem>(this.OnCatalogPurchase), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.MercantileBoosterPurchase, new Action<ClientInventoryUpdateReportItem>(this.OnInventoryUpdateFromPurchase), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.CosmeticPurchase, new Action<ClientInventoryUpdateReportItem>(this.OnInventoryUpdateFromPurchase), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.ModifyPlayerInventory, new Action<ClientInventoryUpdateReportItem>(this.OnModifyPlayerInventory), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.CustomerSupportGrant, new Action<ClientInventoryUpdateReportItem>(this.OnInventoryUpdate), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.OpenChest, new Action<ClientInventoryUpdateReportItem>(this.OnRedeemInventory), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.CampaignGraphPayoutNode, new Action<ClientInventoryUpdateReportItem>(this.OnRedeemInventory), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.AccumulativePayoutNode, new Action<ClientInventoryUpdateReportItem>(this.OnRedeemInventory), publish: false);
          //inventoryManager.Subscribe(InventoryUpdateSource.EventRefundEntry, new Action<ClientInventoryUpdateReportItem>(this.OnEventRefunded));
          //inventoryManager.InventoryUpdated += new Action(this.InventoryManager_InventoryUpdated);
          _subscribedToInventory = true;

          Task.Run(PeriodicUpdater);
        }
        catch (Exception e)
        {
          _logger.Info($"[collection-error]{JsonConvert.SerializeObject(e)}");
        }
      }

    }

    private void PeriodicUpdater()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Inventory != null)
      {
        try
        {
          var collection = WrapperController.Instance.InventoryManager.Cards.Select(pair =>
          {
            return new CardInventoryData
            {
              grpId = pair.Key,
              count = pair.Value,
            };
          }).OrderBy(c => c.grpId).ToArray();

          var collectionEntry = new CollectorEvent
          {
            Attachment = collection,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          WriteToLog("collection", collectionEntry);

          CollectorEvent inventory = new CollectorEvent
          {
            Attachment = WrapperController.Instance.InventoryManager.Inventory,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          WriteToLog("inventory", inventory);
        }
        catch (Exception e)
        {
          _logger.Info($"[collection-error]{JsonConvert.SerializeObject(e)}");
        }
      }

      Thread.Sleep(TimeSpan.FromMinutes(30));
      PeriodicUpdater();
    }

    private void UpdateInventory(ClientInventoryUpdateReportItem payload, string source)
    {
      CollectorEvent inventoryUpdate = new CollectorEvent
      {
        Timestamp = String.Format($"{DateTime.Now:O}"),
        Source = source,
        Attachment = payload
      };

      WriteToLog("inventory-update", inventoryUpdate);
    }

    public void OnDestroy()
    {
      _logger.Info($"[initialization]Shutting down at {System.DateTime.Now:O}. Bye!");
    }

    public void OnDisable()
    {
      _logger.Info($"[initialization]Disabled at {System.DateTime.Now:O}. Oops!");
    }

    public void OnApplicationQuit()
    {
      _logger.Info($"[initialization]App quit at {System.DateTime.Now:O}. Bye bye!");
    }

    private void WriteToLog(string prefix, CollectorEvent collectorEvent)
    {
      _logger.Info($"[{prefix}]{JsonConvert.SerializeObject(collectorEvent)}");
    }
  }
}
