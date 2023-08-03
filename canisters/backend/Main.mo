import Types "Types";
import Core "Core";
import State "State";

import System "System";
import History "History";

shared ({ caller = installer }) actor class Main() {

  let sys = System.IC();
  stable var _state_v0 : State.Stable.State = State.Stable.initialState();
  stable var _history_v0 : History.History = History.init(sys, installer);

  let core = Core.Core(installer, sys, _state_v0, _history_v0);

  public shared ({ caller }) func login() : async Types.Resp.Login {
    core.login(caller);
  };

  public query ({ caller }) func fastLogin() : async ?Types.Resp.Login {
    core.fastLogin(caller);
  };

  public shared ({ caller }) func getEthWallets() : async Types.Resp.GetEthWallets {
    await core.getEthWallets(caller);
  };

  public shared ({ caller }) func connectEthWallet(wallet : Types.EthWallet, signedPrincipal : Types.SignedPrincipal) : async Types.Resp.ConnectEthWallet {
    await core.connectEthWallet(caller, wallet, signedPrincipal);
  };

  public shared ({ caller }) func isNftOwned(caller : Principal, nft : Types.Nft.Nft) : async Bool {
    await core.isNftOwned(caller, nft);
  };

  public shared ({ caller }) func setNfts(nfts : [Types.Nft.Nft]) : async Bool {
    await core.setNfts(caller, nfts);
  };

  public shared ({ caller }) func getHistory() : async ?[History.Event] {
    core.getHistory(caller);
  }

};
